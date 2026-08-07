/*
 * rusteron-media-driver DPDK transport — receive polling and endpoint dispatch
 * (plan §7.6/§7.7).
 *
 * The poller reuses the vendored aeron_udp_transport_poller_t layout: the
 * `transports` array tracks the registered Aeron transports and `bindings_clientd`
 * owns a fixed 4096-entry endpoint map keyed by (destination IPv4, UDP port).
 * poller_add/remove mutate the map outside the hot path; poller_poll drains each
 * distinct role port once per call and dispatches every valid frame to the
 * registered transport via Aeron's recv_func, exactly as a socket recvmmsg
 * would: UDP payload + reconstructed source sockaddr, mbuf recycled immediately
 * after the callback returns.
 *
 * The hot path (poller_poll -> rusteron_dpdk_poller_receive -> classify ->
 * dispatch) performs no heap allocation (mbuf acquisition/recycling excepted),
 * no locking, no logging, no string construction and no syscalls.
 */
/* The poller writes the received-datagram metadata back into the caller's
 * mmsghdr (msg_name, msg_len) exactly as the kernel recvmmsg path does. The
 * complete `struct mmsghdr` is a GNU extension (glibc bits/mmsghdr.h) gated on
 * _GNU_SOURCE; the vendored bindings only forward-declare the tag. This must
 * be defined before any system header is included. */
#define _GNU_SOURCE

#include "rusteron_dpdk_internal.h"

#include <arpa/inet.h>
#include <errno.h>
#include <string.h>
#include <sys/socket.h> /* struct mmsghdr (see above) */

#include "media/aeron_udp_transport_poller.h" /* full poller struct + entry type */
#include "aeron_alloc.h"

#define RUSTERON_DPDK_POLLER_MAX_PORTS 4

/* Private poller state anchored at poller->bindings_clientd. */
typedef struct rusteron_dpdk_poller_state_stct
{
    rusteron_dpdk_endpoint_map_t endpoints;
} rusteron_dpdk_poller_state_t;

/* Reject-class bookkeeping: the rx_stats bucket (existing §7.6 accounting) plus
 * the matching Aeron counter (plan §9). */
static void rusteron_dpdk_rx_count(
    rusteron_dpdk_rx_stats_t *stats, rusteron_dpdk_counters_t *counters, rusteron_dpdk_rx_result_t result)
{
    switch (result)
    {
        case RUSTERON_DPDK_RX_RESULT_IPV6:
            stats->ipv6++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_MULTICAST:
            stats->multicast++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_ETHERTYPE:
            stats->ethertype++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_UNSUPPORTED_ETHERTYPE, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_VLAN:
            stats->vlan++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_IP_OPTIONS:
            stats->ip_options++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_FRAGMENT:
            stats->fragment++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_FRAGMENTED, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_TRUNCATED:
            stats->truncated++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_PROTOCOL:
            stats->protocol++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_UNSUPPORTED_PROTOCOL, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_CHECKSUM:
            stats->checksum++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_CHECKSUM, 1);
            break;
        case RUSTERON_DPDK_RX_RESULT_MULTI_SEGMENT:
            stats->multi_segment++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            break;
        default:
            break;
    }
}

/* Dispatch one parsed frame. The target transport is chosen by the endpoint map
 * (endpoints != NULL) or matched against only_transport's own local endpoint.
 * Rejected frames are counted and dropped; accepted frames invoke recv_func
 * with the UDP payload and a reconstructed source sockaddr_in and return 1. */
static int rusteron_dpdk_dispatch_frame(
    rusteron_dpdk_client_t *client,
    aeron_udp_channel_transport_t *only_transport,
    const rusteron_dpdk_endpoint_map_t *endpoints,
    const rusteron_dpdk_parsed_frame_t *frame,
    struct mmsghdr *msgvec,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    void *clientd)
{
    aeron_udp_channel_transport_t *target = NULL;

    if (NULL != endpoints)
    {
        target = (aeron_udp_channel_transport_t *)rusteron_dpdk_endpoint_map_get(
            endpoints, frame->dst_ip, frame->dst_port);
    }
    else if (frame->dst_ip == client->port->local_ip &&
             frame->dst_port == client->local_udp_port)
    {
        target = only_transport;
    }

    if (NULL == target)
    {
        rusteron_dpdk_counters_t *counters = &client->port->counters;
        if (frame->dst_ip != client->port->local_ip)
        {
            client->runtime->rx_stats.foreign_dst++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
        }
        else
        {
            client->runtime->rx_stats.unknown_port++;
            rusteron_dpdk_counters_add(counters, RD_COUNTER_DISCARD, 1);
            rusteron_dpdk_counters_add(counters, RD_COUNTER_QUEUE_DROP, 1);
        }
        return 0;
    }

    /* Reconstruct the source sockaddr the kernel would have returned from a
     * recvmsg on the matching UDP socket (plan §7.6). */
    struct sockaddr_storage storage;
    struct sockaddr_in *sin = (struct sockaddr_in *)&storage;
    memset(&storage, 0, sizeof(storage));
    sin->sin_family = AF_INET;
    sin->sin_port = htons(frame->src_port);
    sin->sin_addr.s_addr = frame->src_ip;

    struct sockaddr_storage *addr = (struct sockaddr_storage *)msgvec[0].msg_hdr.msg_name;
    struct sockaddr_storage local_addr;
    if (NULL == addr)
    {
        addr = &local_addr;
    }
    memcpy(addr, &storage, sizeof(storage));
    msgvec[0].msg_len = (unsigned int)frame->payload_len;

    recv_func(
        target->data_paths,
        target,
        clientd,
        target->dispatch_clientd,
        target->destination_clientd,
        (uint8_t *)frame->payload,
        frame->payload_len,
        addr,
        NULL);

    *bytes_rcved += (int64_t)frame->payload_len;
    client->runtime->rx_stats.accepted++;
    rusteron_dpdk_counters_add(&client->port->counters, RD_COUNTER_RX_PKTS, 1);
    rusteron_dpdk_counters_add(&client->port->counters, RD_COUNTER_RX_BYTES, (int64_t)frame->payload_len);
    return 1;
}

int rusteron_dpdk_poller_receive(
    rusteron_dpdk_client_t *client,
    aeron_udp_channel_transport_t *only_transport,
    const rusteron_dpdk_endpoint_map_t *endpoints,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    void *clientd)
{
    if (NULL == client || NULL == client->runtime || NULL == client->port ||
        NULL == msgvec || NULL == bytes_rcved || NULL == recv_func)
    {
        return -1;
    }

    rusteron_dpdk_transport_t *native = client->runtime;
    rusteron_dpdk_port_t *port = client->port;
    const rusteron_dpdk_port_ops_t *ops = native->ops;

    /* Poll no more than min(Aeron vlen, configured burst size) mbufs per call
     * (plan §7.6). */
    size_t limit = vlen < native->config.burst_size ? vlen : native->config.burst_size;
    if (0 == limit)
    {
        return 0;
    }
    if (limit > 256)
    {
        limit = 256;
    }

    rusteron_dpdk_mbuf_t views[256];
    rusteron_dpdk_mbuf_t *batch[256];
    for (size_t i = 0; i < limit; i++)
    {
        batch[i] = &views[i];
    }

    uint16_t received = ops->rx_burst(port->port_id, port->rx_queue_id, batch, (uint16_t)limit);

    int work_count = 0;
    for (uint16_t i = 0; i < received; i++)
    {
        rusteron_dpdk_mbuf_t *m = &views[i];
        rusteron_dpdk_parsed_frame_t frame;
        rusteron_dpdk_rx_result_t result = rusteron_dpdk_packet_classify_rx(m, &frame);

        RD_DEBUG("rx: frame len=%u result=%d ether=0x%02x%02x on %s\n",
                 m->frame_len, (int)result, m->data[12], m->data[13], port->pci);

        rusteron_dpdk_counters_add(&port->counters, RD_COUNTER_POLLER, 1);

        if (RUSTERON_DPDK_RX_RESULT_OK == result)
        {
            work_count += rusteron_dpdk_dispatch_frame(
                client, only_transport, endpoints, &frame, msgvec,
                bytes_rcved, recv_func, clientd);
        }
        else if (RUSTERON_DPDK_RX_RESULT_ARP == result)
        {
            native->rx_stats.arp++;
            rusteron_dpdk_arp_handle_frame(&native->arp, native, port, m->data, m->frame_len);
        }
        else
        {
            rusteron_dpdk_rx_count(&native->rx_stats, &port->counters, result);
        }

        /* The callback has returned (and the ARP handler has run); the mbuf is
         * recycled now and no later (plan §7.6). */
        ops->mbuf_release(m);
    }

    rusteron_dpdk_counters_sample(&port->counters, port, ops);
    return work_count;
}

int rusteron_dpdk_poller_init(
    aeron_udp_transport_poller_t *poller,
    aeron_driver_context_t *context,
    aeron_udp_channel_transport_affinity_t affinity)
{
    (void)context;
    (void)affinity;

    if (NULL == poller)
    {
        rusteron_dpdk_set_error("poller_init: poller must not be NULL");
        return -1;
    }

    poller->transports.array = NULL;
    poller->transports.length = 0;
    poller->transports.capacity = 0;
    poller->fd = -1;

    rusteron_dpdk_poller_state_t *state = NULL;
    if (aeron_alloc((void **)&state, sizeof(rusteron_dpdk_poller_state_t)) < 0)
    {
        rusteron_dpdk_set_error("poller_init: out of memory allocating poller state");
        return -1;
    }
    if (rusteron_dpdk_endpoint_map_init(&state->endpoints) < 0)
    {
        aeron_free(state);
        rusteron_dpdk_set_error("poller_init: out of memory allocating endpoint map");
        return -1;
    }
    poller->bindings_clientd = state;
    return 0;
}

int rusteron_dpdk_poller_close(aeron_udp_transport_poller_t *poller)
{
    if (NULL == poller)
    {
        return 0;
    }

    rusteron_dpdk_poller_state_t *state = (rusteron_dpdk_poller_state_t *)poller->bindings_clientd;
    if (NULL != state)
    {
        rusteron_dpdk_endpoint_map_close(&state->endpoints);
        aeron_free(state);
        poller->bindings_clientd = NULL;
    }
    if (NULL != poller->transports.array)
    {
        aeron_free(poller->transports.array);
    }
    poller->transports.array = NULL;
    poller->transports.length = 0;
    poller->transports.capacity = 0;
    poller->fd = -1;
    return 0;
}

int rusteron_dpdk_poller_add(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    if (NULL == poller || NULL == transport)
    {
        rusteron_dpdk_set_error("poller_add: poller/transport must not be NULL");
        return -1;
    }
    rusteron_dpdk_poller_state_t *state = (rusteron_dpdk_poller_state_t *)poller->bindings_clientd;
    if (NULL == state)
    {
        rusteron_dpdk_set_error("poller_add: poller is not initialized");
        return -1;
    }
    rusteron_dpdk_client_t *client = transport->bindings_clientd;
    if (NULL == client || NULL == client->port)
    {
        rusteron_dpdk_set_error("poller_add: transport has no DPDK client state");
        return -1;
    }

    /* Grow the registered-transport list before touching the endpoint map so a
     * failure leaves no half-registered endpoint behind. */
    if (poller->transports.length == poller->transports.capacity)
    {
        size_t new_capacity = poller->transports.capacity == 0 ? 4 : poller->transports.capacity * 2;
        void *array = poller->transports.array;
        if (aeron_reallocf(&array, new_capacity * sizeof(aeron_udp_channel_transport_entry_t)) < 0)
        {
            poller->transports.array = NULL;
            rusteron_dpdk_set_error("poller_add: out of memory growing transport list");
            return -1;
        }
        poller->transports.array = (aeron_udp_channel_transport_entry_t *)array;
        poller->transports.capacity = new_capacity;
    }

    if (rusteron_dpdk_endpoint_map_put(
            &state->endpoints, client->port->local_ip, client->local_udp_port, transport) < 0)
    {
        AERON_SET_ERR(EADDRINUSE, "poller_add: endpoint %s:%u is already registered",
                      client->port->local_ipv4, (unsigned)client->local_udp_port);
        rusteron_dpdk_set_error("poller_add: duplicate endpoint");
        return -1;
    }

    poller->transports.array[poller->transports.length++].transport = transport;
    return 0;
}

int rusteron_dpdk_poller_remove(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    if (NULL == poller || NULL == transport)
    {
        return -1;
    }
    rusteron_dpdk_poller_state_t *state = (rusteron_dpdk_poller_state_t *)poller->bindings_clientd;

    int index = -1;
    for (size_t i = 0; i < poller->transports.length; i++)
    {
        if (poller->transports.array[i].transport == transport)
        {
            index = (int)i;
            break;
        }
    }
    if (index < 0)
    {
        return 0; /* not registered */
    }

    size_t last = poller->transports.length - 1;
    if ((size_t)index < last)
    {
        poller->transports.array[index] = poller->transports.array[last];
    }
    poller->transports.length--;

    if (NULL != state)
    {
        rusteron_dpdk_client_t *client = transport->bindings_clientd;
        if (NULL != client && NULL != client->port)
        {
            rusteron_dpdk_endpoint_map_remove(
                &state->endpoints, client->port->local_ip, client->local_udp_port);
        }
    }
    return 0;
}

int rusteron_dpdk_poller_poll(
    aeron_udp_transport_poller_t *poller,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    aeron_udp_channel_transport_recvmmsg_func_t recvmmsg_func,
    void *clientd)
{
    (void)recvmmsg_func;

    if (NULL == poller || NULL == msgvec || NULL == bytes_rcved || NULL == recv_func)
    {
        return -1;
    }
    rusteron_dpdk_poller_state_t *state = (rusteron_dpdk_poller_state_t *)poller->bindings_clientd;
    if (NULL == state || 0 == poller->transports.length)
    {
        return 0;
    }

    int work_count = 0;

    /* A role ENA is shared by every transport of that affinity, so drain each
     * distinct port at most once per poll call (plan §7.6). */
    uint16_t seen_ports[RUSTERON_DPDK_POLLER_MAX_PORTS];
    size_t seen_count = 0;

    for (size_t i = 0; i < poller->transports.length; i++)
    {
        aeron_udp_channel_transport_t *transport = poller->transports.array[i].transport;
        if (NULL == transport)
        {
            continue;
        }
        rusteron_dpdk_client_t *client = transport->bindings_clientd;
        if (NULL == client || NULL == client->port)
        {
            continue;
        }
        uint16_t port_id = client->port->port_id;

        int already = 0;
        for (size_t s = 0; s < seen_count; s++)
        {
            if (seen_ports[s] == port_id)
            {
                already = 1;
                break;
            }
        }
        if (already)
        {
            continue;
        }
        if (seen_count < RUSTERON_DPDK_POLLER_MAX_PORTS)
        {
            seen_ports[seen_count++] = port_id;
        }

        int received = rusteron_dpdk_poller_receive(
            client, NULL, &state->endpoints, msgvec, vlen, bytes_rcved, recv_func, clientd);
        if (received < 0)
        {
            return -1;
        }
        work_count += received;
    }

    return work_count;
}
