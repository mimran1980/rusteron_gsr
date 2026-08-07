/*
 * rusteron-media-driver DPDK ENA kernel-bypass transport — stable ABI.
 *
 * Ticket 1 delivers the ABI and the populated Aeron transport binding table.
 * The per-callback behaviour lands in later tickets (EAL/port init in
 * Ticket 3, transmit in Ticket 4, receive polling in Ticket 5). Until then
 * the callbacks are live stubs: they record a "not implemented" error in the
 * thread-local last_error buffer and return a failure so a driver that binds
 * them early fails loudly instead of silently dropping traffic.
 */
#include "rusteron_dpdk_internal.h"

#include <arpa/inet.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "util/aeron_error.h"
#include "media/aeron_udp_channel_transport.h"
/* Full driver-context layout: the counters manager is reached via
 * context->counters_manager at transport init (plan §9). bindings.h only
 * forward-declares the type. */
#include "aeron_driver_context.h"

#define RUSTERON_DPDK_BINDINGS_NAME "rusteron-dpdk-ena"
#define RUSTERON_DPDK_BINDINGS_TYPE "media"

/* Aeron never passes more than AERON_NETWORK_PUBLICATION_MAX_MESSAGES_PER_SEND
 * iovecs to one send; the batch buffer must hold a full burst regardless. */
#define RUSTERON_DPDK_MAX_MESSAGES_PER_SEND 16

/* The transport layout lives in rusteron_dpdk_internal.h (Ticket 3). */

/* Process-lifetime native runtime. EAL can be initialized exactly once, so
 * there is at most one live transport; the binding callbacks find it here. */
static rusteron_dpdk_transport_t *rusteron_dpdk_the_runtime = NULL;

rusteron_dpdk_transport_t *rusteron_dpdk_active_runtime(void)
{
    return rusteron_dpdk_the_runtime;
}

static _Thread_local char rusteron_dpdk_error_buffer[1024];
static _Thread_local int rusteron_dpdk_error_code = RUSTERON_DPDK_ERR_OK;

void rusteron_dpdk_set_error_code(const char *message, int code)
{
    strncpy(rusteron_dpdk_error_buffer, message, sizeof(rusteron_dpdk_error_buffer) - 1);
    rusteron_dpdk_error_buffer[sizeof(rusteron_dpdk_error_buffer) - 1] = '\0';
    rusteron_dpdk_error_code = code;
}

void rusteron_dpdk_set_error(const char *message)
{
    rusteron_dpdk_set_error_code(message, RUSTERON_DPDK_ERR_NATIVE);
}

/*
 * Minimal string field validation shared by config fields that hold fixed
 * buffers. Strings must be non-empty and NUL-terminated within the buffer.
 */
static int rusteron_dpdk_config_validate_string(const char *value, size_t capacity, const char *name)
{
    if (NULL == value || '\0' == value[0])
    {
        char message[256];
        snprintf(message, sizeof(message), "config field '%s' must be a non-empty string", name);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    if (memchr(value, '\0', capacity) == NULL)
    {
        char message[256];
        snprintf(message, sizeof(message), "config field '%s' is not NUL-terminated within %zu bytes", name, capacity);
        rusteron_dpdk_set_error(message);
        return -1;
    }

    return 0;
}

static int rusteron_dpdk_config_validate(const rusteron_dpdk_config_t *config)
{
    if (NULL == config)
    {
        rusteron_dpdk_set_error("config must not be NULL");
        return -1;
    }

    if (config->struct_size != RUSTERON_DPDK_CONFIG_STRUCT_SIZE)
    {
        rusteron_dpdk_set_error("config.struct_size does not match the native layout — Rust/native ABI mismatch");
        return -1;
    }

    if (rusteron_dpdk_config_validate_string(config->file_prefix, sizeof(config->file_prefix), "file_prefix") < 0 ||
        rusteron_dpdk_config_validate_string(config->hugepage_dir, sizeof(config->hugepage_dir), "hugepage_dir") < 0 ||
        rusteron_dpdk_config_validate_string(config->sender_pci, sizeof(config->sender_pci), "sender_pci") < 0 ||
        rusteron_dpdk_config_validate_string(config->sender_ipv4, sizeof(config->sender_ipv4), "sender_ipv4") < 0 ||
        rusteron_dpdk_config_validate_string(config->sender_gateway, sizeof(config->sender_gateway), "sender_gateway") < 0 ||
        rusteron_dpdk_config_validate_string(config->receiver_pci, sizeof(config->receiver_pci), "receiver_pci") < 0 ||
        rusteron_dpdk_config_validate_string(config->receiver_ipv4, sizeof(config->receiver_ipv4), "receiver_ipv4") < 0 ||
        rusteron_dpdk_config_validate_string(config->receiver_gateway, sizeof(config->receiver_gateway), "receiver_gateway") < 0)
    {
        return -1;
    }

    if (config->sender_prefix_len < 1 || config->sender_prefix_len > 32 ||
        config->receiver_prefix_len < 1 || config->receiver_prefix_len > 32)
    {
        rusteron_dpdk_set_error("prefix lengths must be in 1..=32");
        return -1;
    }

    if (config->burst_size < 1 || config->burst_size > 256)
    {
        rusteron_dpdk_set_error("burst_size must be in 1..=256");
        return -1;
    }

    if (config->rx_descriptors < 64 || config->rx_descriptors > 8192 ||
        config->tx_descriptors < 64 || config->tx_descriptors > 8192)
    {
        rusteron_dpdk_set_error("rx/tx descriptors must be in 64..=8192");
        return -1;
    }

    if (config->max_aeron_mtu == 0 || config->max_aeron_mtu > 1472 || (config->max_aeron_mtu % 32) != 0)
    {
        rusteron_dpdk_set_error("max_aeron_mtu must be non-zero, 32-aligned, and <= 1472");
        return -1;
    }

    if (config->mbufs_per_port == 0)
    {
        rusteron_dpdk_set_error("mbufs_per_port must be non-zero");
        return -1;
    }

    return 0;
}

int rusteron_dpdk_transport_create(
    const rusteron_dpdk_config_t *config,
    rusteron_dpdk_transport_t **transport)
{
    if (NULL == transport)
    {
        rusteron_dpdk_set_error("transport out-param must not be NULL");
        return -1;
    }

    *transport = NULL;

    if (rusteron_dpdk_config_validate(config) < 0)
    {
        return -1;
    }

    rusteron_dpdk_transport_t *native = calloc(1, sizeof(rusteron_dpdk_transport_t));
    if (NULL == native)
    {
        rusteron_dpdk_set_error("out of memory allocating DPDK transport");
        return -1;
    }

    native->config = *config;

    if (rusteron_dpdk_runtime_init(native) < 0)
    {
        free(native);
        return -1;
    }

    rusteron_dpdk_the_runtime = native;
    *transport = native;
    return 0;
}

int rusteron_dpdk_transport_install(
    rusteron_dpdk_transport_t *transport,
    aeron_driver_context_t *context)
{
    if (NULL == transport)
    {
        rusteron_dpdk_set_error("transport must not be NULL");
        return -1;
    }

    if (NULL == context)
    {
        rusteron_dpdk_set_error("context must not be NULL");
        return -1;
    }

    /* Only the sender/receiver bindings are replaced; the conductor's
     * resolver bindings stay on the socket media so DNS resolution keeps
     * working over the kernel. */
    context->udp_channel_transport_bindings = rusteron_dpdk_transport_bindings();
    return 0;
}

int rusteron_dpdk_transport_close(rusteron_dpdk_transport_t *transport)
{
    if (NULL == transport)
    {
        return 0;
    }

    if (rusteron_dpdk_the_runtime == transport)
    {
        rusteron_dpdk_the_runtime = NULL;
    }
    rusteron_dpdk_runtime_cleanup(transport);
    free(transport);
    return 0;
}

/* Test reset: drop the singleton so a fresh transport can be created in the
 * next test after the EAL guard was cleared by rusteron_dpdk_test_reset(). */
void rusteron_dpdk_transport_test_reset(void)
{
    rusteron_dpdk_the_runtime = NULL;
}

/* --- Aeron transport binding callbacks ------------------------------------ */

static int rusteron_dpdk_not_implemented(const char *what)
{
    char message[256];
    snprintf(message, sizeof(message), "DPDK transport %s not implemented (Ticket 3+ delivers it)", what);
    rusteron_dpdk_set_error(message);
    return -1;
}

static int rusteron_dpdk_transport_init(
    aeron_udp_channel_transport_t *transport,
    struct sockaddr_storage *bind_addr,
    struct sockaddr_storage *multicast_if_addr,
    struct sockaddr_storage *connect_addr,
    aeron_udp_channel_transport_params_t *params,
    aeron_driver_context_t *context,
    aeron_udp_channel_transport_affinity_t affinity)
{
    (void)multicast_if_addr;

    if (NULL == bind_addr)
    {
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport init: bind_addr is NULL");
        rusteron_dpdk_set_error("DPDK transport init: bind_addr must not be NULL");
        return -1;
    }
    if (AF_INET != bind_addr->ss_family)
    {
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport supports IPv4 unicast only");
        rusteron_dpdk_set_error("DPDK transport init: only IPv4 bind addresses are supported");
        return -1;
    }
    if (NULL == params || 0 == params->mtu_length)
    {
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport init: mtu_length must be greater than 0");
        rusteron_dpdk_set_error("DPDK transport init: mtu_length must be greater than 0");
        return -1;
    }

    rusteron_dpdk_transport_t *native = rusteron_dpdk_active_runtime();
    if (NULL == native)
    {
        AERON_SET_ERR(ENODEV, "%s", "DPDK transport init: no native runtime (rusteron_dpdk_transport_create was not called)");
        rusteron_dpdk_set_error("DPDK transport init: no native runtime");
        return -1;
    }

    rusteron_dpdk_port_t *port;
    switch (affinity)
    {
        case AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_SENDER:
            port = &native->sender;
            break;
        case AERON_UDP_CHANNEL_TRANSPORT_AFFINITY_RECEIVER:
            port = &native->receiver;
            break;
        default:
            AERON_SET_ERR(EINVAL, "DPDK transport init: unsupported affinity %d", (int)affinity);
            rusteron_dpdk_set_error("DPDK transport init: conductor affinity is not a data-path transport");
            return -1;
    }
    if (!port->link_up)
    {
        AERON_SET_ERR(ENODEV, "DPDK transport init: role ENA (port %u) is not up", port->port_id);
        rusteron_dpdk_set_error("DPDK transport init: role ENA is not up");
        return -1;
    }

    /* Surface the port's counters into the driver's counters manager (plan §9).
     * Best effort: a NULL context (unit tests) or a NULL counters manager skips
     * registration and the counters bumps become no-ops. */
    rusteron_dpdk_counters_register(
        &port->counters,
        NULL != context ? context->counters_manager : NULL,
        port, native->ops);
    rusteron_dpdk_counters_add(&port->counters, RD_COUNTER_TRANSPORT, 1);

    rusteron_dpdk_client_t *client = calloc(1, sizeof(rusteron_dpdk_client_t));
    if (NULL == client)
    {
        rusteron_dpdk_set_error("DPDK transport init: out of memory allocating client state");
        return -1;
    }
    client->runtime = native;
    client->port = port;
    client->affinity = (int)affinity;
    client->local_udp_port = (uint16_t)ntohs(((struct sockaddr_in *)bind_addr)->sin_port);
    client->mtu = params->mtu_length;
    if (client->mtu > native->config.max_aeron_mtu)
    {
        /* The device L3 MTU caps every frame; clamp to it so an over-large
         * channel MTU degrades to "oversized datagram rejected" instead of
         * handing the NIC a frame bigger than the configured device MTU. */
        client->mtu = native->config.max_aeron_mtu;
    }

    transport->bindings_clientd = client;
    /* The connect address is caller-owned; keep a copy so NULL-address sends
     * survive past the caller's frame (connected transport, plan §7.4). */
    transport->connected_address = NULL;
    if (NULL != connect_addr)
    {
        memcpy(&client->connected_address, connect_addr, sizeof(client->connected_address));
        transport->connected_address = &client->connected_address;
    }
    transport->fd = -1; /* the DPDK transport owns no socket */
    transport->recv_fd = -1;

    return 0;
}

static int rusteron_dpdk_transport_reconnect(
    aeron_udp_channel_transport_t *transport,
    struct sockaddr_storage *connect_addr)
{
    if (NULL != connect_addr)
    {
        rusteron_dpdk_client_t *client = transport->bindings_clientd;
        if (NULL == client)
        {
            return -1;
        }
        memcpy(&client->connected_address, connect_addr, sizeof(client->connected_address));
        transport->connected_address = &client->connected_address;
    }
    return 0;
}

static int rusteron_dpdk_transport_close_transport(aeron_udp_channel_transport_t *transport)
{
    if (NULL == transport)
    {
        return 0;
    }
    rusteron_dpdk_client_t *client = transport->bindings_clientd;
    if (NULL != client && NULL != client->port)
    {
        rusteron_dpdk_counters_add(&client->port->counters, RD_COUNTER_TRANSPORT, -1);
    }
    transport->bindings_clientd = NULL;
    transport->connected_address = NULL;
    transport->fd = -1;
    transport->recv_fd = -1;
    free(client);
    return 0;
}

/* Aeron drives a single registered transport through recvmmsg; the shared
 * receive loop dispatches only frames addressed to this transport's own
 * endpoint (plan §7.6). */
static int rusteron_dpdk_transport_recvmmsg(
    aeron_udp_channel_transport_t *transport,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    void *clientd)
{
    rusteron_dpdk_client_t *client = transport != NULL ? transport->bindings_clientd : NULL;
    if (NULL == client)
    {
        return 0;
    }
    return rusteron_dpdk_poller_receive(
        client, transport, NULL, msgvec, vlen, bytes_rcved, recv_func, clientd);
}

static int rusteron_dpdk_transport_send(
    aeron_udp_channel_data_paths_t *data_paths,
    aeron_udp_channel_transport_t *transport,
    struct sockaddr_storage *address,
    struct iovec *iov,
    size_t iov_length,
    int64_t *bytes_sent)
{
    (void)data_paths;

    rusteron_dpdk_client_t *client = transport->bindings_clientd;
    if (NULL == client || NULL == client->runtime || NULL == client->port)
    {
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: transport has no client state");
        rusteron_dpdk_set_error("DPDK transport send: transport has no client state");
        return -1;
    }
    if (NULL == bytes_sent || (NULL == iov && iov_length > 0))
    {
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: iov/bytes_sent must not be NULL");
        rusteron_dpdk_set_error("DPDK transport send: iov/bytes_sent must not be NULL");
        return -1;
    }
    if (0 == iov_length)
    {
        return 0;
    }

    rusteron_dpdk_transport_t *native = client->runtime;
    rusteron_dpdk_port_t *port = client->port;
    const rusteron_dpdk_port_ops_t *ops = native->ops;
    rusteron_dpdk_counters_t *counters = &port->counters;

    struct sockaddr_storage *dst = address != NULL ? address : transport->connected_address;
    if (NULL == dst)
    {
        rusteron_dpdk_counters_add(counters, RD_COUNTER_ERROR, 1);
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: no destination address");
        rusteron_dpdk_set_error("DPDK transport send: no destination address");
        return -1;
    }
    if (AF_INET != dst->ss_family)
    {
        rusteron_dpdk_counters_add(counters, RD_COUNTER_ERROR, 1);
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: only IPv4 unicast destinations are supported");
        rusteron_dpdk_set_error("DPDK transport send: only IPv4 unicast destinations are supported");
        return -1;
    }
    struct sockaddr_in *sin = (struct sockaddr_in *)dst;
    uint32_t dst_ip = sin->sin_addr.s_addr;
    uint16_t dst_port = (uint16_t)ntohs(sin->sin_port);

    if (rusteron_dpdk_ipv4_is_multicast(dst_ip))
    {
        rusteron_dpdk_counters_add(counters, RD_COUNTER_ERROR, 1);
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: multicast destinations are not supported");
        rusteron_dpdk_set_error("DPDK transport send: multicast destinations are not supported");
        return -1;
    }

    /* Route: an in-subnet next hop is ARP'd directly; anything else goes via
     * the configured gateway while keeping the destination IP in the header. */
    uint32_t net = rusteron_dpdk_ipv4_mask(port->prefix_len);
    uint32_t next_hop = ((dst_ip & net) == (port->local_ip & net)) ? dst_ip : port->gateway_ip;

    uint8_t dst_mac[RUSTERON_DPDK_ETH_ADDR_LEN];
    if (!rusteron_dpdk_arp_resolve(&native->arp, native, port, next_hop, dst_mac))
    {
        rusteron_dpdk_counters_add(counters, RD_COUNTER_ARP_MISS, 1);
        return 0; /* ARP request in flight; Aeron retries the zero result */
    }

    rusteron_dpdk_mbuf_t views[256];
    rusteron_dpdk_mbuf_t *batch[256];
    size_t batch_len = 0;
    size_t sent = 0;
    int oversized = 0;

    /* Every iovec is one datagram (plan §7.4); burst_size only sizes each
     * rte_eth_tx_burst flush, so consume the whole list. */
    for (size_t i = 0; i < iov_length; i++)
    {
        if (iov[i].iov_len > client->mtu)
        {
            oversized = 1;
            break;
        }

        rusteron_dpdk_mbuf_t *m = &views[batch_len];
        if (ops->mbuf_alloc(port->mempool, m) < 0)
        {
            rusteron_dpdk_counters_add(counters, RD_COUNTER_NOBUFS, 1);
            break; /* mbuf exhaustion: flush the already-built prefix below */
        }
        if (rusteron_dpdk_packet_build_udp(
                m, dst_mac, port->mac, dst_ip, port->local_ip,
                client->local_udp_port, dst_port, &iov[i], 1, client->mtu) < 0)
        {
            ops->mbuf_release(m);
            oversized = 1;
            break;
        }
        batch[batch_len] = m;
        batch_len++;

        if (batch_len == native->config.burst_size)
        {
            uint16_t bs = ops->tx_burst(port->port_id, port->tx_queue_id, batch, (uint16_t)batch_len);
            sent += bs;
            batch_len = 0;
            if (bs < native->config.burst_size)
            {
                rusteron_dpdk_counters_add(counters, RD_COUNTER_TX_EAGAIN, 1);
                break; /* NIC backpressure: stop, report the accepted prefix */
            }
        }
    }
    if (batch_len > 0)
    {
        uint16_t bs = ops->tx_burst(port->port_id, port->tx_queue_id, batch, (uint16_t)batch_len);
        if (bs < (uint16_t)batch_len)
        {
            rusteron_dpdk_counters_add(counters, RD_COUNTER_TX_EAGAIN, 1);
        }
        sent += bs;
        batch_len = 0;
    }

    /* bytes_sent accrues only for the accepted prefix (plan §7.4). */
    for (size_t i = 0; i < sent; i++)
    {
        *bytes_sent += iov[i].iov_len;
    }
    rusteron_dpdk_counters_add(counters, RD_COUNTER_TX_PKTS, (int64_t)sent);
    rusteron_dpdk_counters_add(counters, RD_COUNTER_TX_BYTES, *bytes_sent);

    if (oversized)
    {
        rusteron_dpdk_counters_add(counters, RD_COUNTER_ERROR, 1);
        /* The prefix was sent and accounted; the oversized datagram itself is a
         * permanent validation error surfaced through the Aeron mechanism. */
        AERON_SET_ERR(EINVAL, "%s", "DPDK transport send: datagram exceeds the channel MTU");
        rusteron_dpdk_set_error("DPDK transport send: oversized datagram rejected");
        return -1;
    }

    rusteron_dpdk_counters_sample(counters, port, ops);
    return (int)sent;
}

static int rusteron_dpdk_transport_get_so_rcvbuf(
    aeron_udp_channel_transport_t *transport, size_t *so_rcvbuf)
{
    (void)transport;
    if (NULL == so_rcvbuf)
    {
        rusteron_dpdk_set_error("so_rcvbuf out-param must not be NULL");
        return -1;
    }

    *so_rcvbuf = 0;
    return 0;
}

static int rusteron_dpdk_transport_bind_addr_and_port(
    aeron_udp_channel_transport_t *transport, char *buffer, size_t length)
{
    if (NULL == buffer || length == 0)
    {
        rusteron_dpdk_set_error("bind_addr_and_port buffer must not be NULL/empty");
        return -1;
    }

    rusteron_dpdk_client_t *client = transport->bindings_clientd;
    if (NULL == client || NULL == client->port)
    {
        rusteron_dpdk_set_error("DPDK transport bind_addr_and_port: no client state");
        return -1;
    }

    /* Same "ip:port" shape aeron_format_source_identity produces for IPv4. */
    snprintf(buffer, length, "%s:%u", client->port->local_ipv4, (unsigned)client->local_udp_port);
    return 0;
}

/* The poller callbacks live in rusteron_dpdk_poller.c (plan §7.6/§7.7). */
static int rusteron_dpdk_transport_poller_init(
    aeron_udp_transport_poller_t *poller,
    aeron_driver_context_t *context,
    aeron_udp_channel_transport_affinity_t affinity)
{
    return rusteron_dpdk_poller_init(poller, context, affinity);
}

static int rusteron_dpdk_transport_poller_close(aeron_udp_transport_poller_t *poller)
{
    return rusteron_dpdk_poller_close(poller);
}

static int rusteron_dpdk_transport_poller_add(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    return rusteron_dpdk_poller_add(poller, transport);
}

static int rusteron_dpdk_transport_poller_remove(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    return rusteron_dpdk_poller_remove(poller, transport);
}

static int rusteron_dpdk_transport_poller_poll(
    aeron_udp_transport_poller_t *poller,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    aeron_udp_channel_transport_recvmmsg_func_t recvmmsg_func,
    void *clientd)
{
    return rusteron_dpdk_poller_poll(
        poller, msgvec, vlen, bytes_rcved, recv_func, recvmmsg_func, clientd);
}

/* --- binding table --------------------------------------------------------- */

static aeron_udp_channel_transport_bindings_t rusteron_dpdk_transport_bindings_instance =
    {
        .init_func = rusteron_dpdk_transport_init,
        .reconnect_func = rusteron_dpdk_transport_reconnect,
        .close_func = rusteron_dpdk_transport_close_transport,
        .recvmmsg_func = rusteron_dpdk_transport_recvmmsg,
        .send_func = rusteron_dpdk_transport_send,
        .get_so_rcvbuf_func = rusteron_dpdk_transport_get_so_rcvbuf,
        .bind_addr_and_port_func = rusteron_dpdk_transport_bind_addr_and_port,
        .poller_init_func = rusteron_dpdk_transport_poller_init,
        .poller_close_func = rusteron_dpdk_transport_poller_close,
        .poller_add_func = rusteron_dpdk_transport_poller_add,
        .poller_remove_func = rusteron_dpdk_transport_poller_remove,
        .poller_poll_func = rusteron_dpdk_transport_poller_poll,
        .meta_info =
            {
                .name = RUSTERON_DPDK_BINDINGS_NAME,
                .type = RUSTERON_DPDK_BINDINGS_TYPE,
                .source_symbol = NULL,
            },
    };

aeron_udp_channel_transport_bindings_t *rusteron_dpdk_transport_bindings(void)
{
    rusteron_dpdk_transport_bindings_instance.meta_info.source_symbol = &rusteron_dpdk_transport_bindings_instance;
    return &rusteron_dpdk_transport_bindings_instance;
}

const char *rusteron_dpdk_last_error(void)
{
    if ('\0' == rusteron_dpdk_error_buffer[0])
    {
        return "no error";
    }

    return rusteron_dpdk_error_buffer;
}

int rusteron_dpdk_last_error_code(void)
{
    return rusteron_dpdk_error_code;
}
