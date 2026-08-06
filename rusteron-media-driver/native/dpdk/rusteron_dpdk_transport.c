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
#include "rusteron_dpdk_transport.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "util/aeron_error.h"

#define RUSTERON_DPDK_BINDINGS_NAME "rusteron-dpdk-ena"
#define RUSTERON_DPDK_BINDINGS_TYPE "media"

struct rusteron_dpdk_transport_stct
{
    rusteron_dpdk_config_t config;
    /* DPDK runtime state (EAL, ports, mempools) is added in Ticket 3. */
};

static _Thread_local char rusteron_dpdk_error_buffer[1024];

static void rusteron_dpdk_set_error(const char *message)
{
    strncpy(rusteron_dpdk_error_buffer, message, sizeof(rusteron_dpdk_error_buffer) - 1);
    rusteron_dpdk_error_buffer[sizeof(rusteron_dpdk_error_buffer) - 1] = '\0';
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

    free(transport);
    return 0;
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
    (void)transport;
    (void)bind_addr;
    (void)multicast_if_addr;
    (void)connect_addr;
    (void)params;
    (void)context;
    (void)affinity;
    return rusteron_dpdk_not_implemented("init");
}

static int rusteron_dpdk_transport_reconnect(
    aeron_udp_channel_transport_t *transport,
    struct sockaddr_storage *connect_addr)
{
    (void)transport;
    (void)connect_addr;
    return rusteron_dpdk_not_implemented("reconnect");
}

static int rusteron_dpdk_transport_close_transport(aeron_udp_channel_transport_t *transport)
{
    (void)transport;
    return 0;
}

static int rusteron_dpdk_transport_recvmmsg(
    aeron_udp_channel_transport_t *transport,
    struct mmsghdr *msgvec,
    size_t vlen,
    int64_t *bytes_rcved,
    aeron_udp_transport_recv_func_t recv_func,
    void *clientd)
{
    (void)transport;
    (void)msgvec;
    (void)vlen;
    (void)bytes_rcved;
    (void)recv_func;
    (void)clientd;
    return rusteron_dpdk_not_implemented("recvmmsg");
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
    (void)transport;
    (void)address;
    (void)iov;
    (void)iov_length;
    (void)bytes_sent;
    return rusteron_dpdk_not_implemented("send");
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
    (void)transport;
    if (NULL == buffer || length == 0)
    {
        rusteron_dpdk_set_error("bind_addr_and_port buffer must not be NULL/empty");
        return -1;
    }

    buffer[0] = '\0';
    return rusteron_dpdk_not_implemented("bind_addr_and_port");
}

static int rusteron_dpdk_transport_poller_init(
    aeron_udp_transport_poller_t *poller,
    aeron_driver_context_t *context,
    aeron_udp_channel_transport_affinity_t affinity)
{
    (void)poller;
    (void)context;
    (void)affinity;
    return rusteron_dpdk_not_implemented("poller_init");
}

static int rusteron_dpdk_transport_poller_close(aeron_udp_transport_poller_t *poller)
{
    (void)poller;
    return 0;
}

static int rusteron_dpdk_transport_poller_add(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    (void)poller;
    (void)transport;
    return rusteron_dpdk_not_implemented("poller_add");
}

static int rusteron_dpdk_transport_poller_remove(
    aeron_udp_transport_poller_t *poller, aeron_udp_channel_transport_t *transport)
{
    (void)poller;
    (void)transport;
    return 0;
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
    (void)poller;
    (void)msgvec;
    (void)vlen;
    (void)bytes_rcved;
    (void)recv_func;
    (void)recvmmsg_func;
    (void)clientd;
    return rusteron_dpdk_not_implemented("poller_poll");
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
