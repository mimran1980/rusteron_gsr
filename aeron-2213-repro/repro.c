/*
 * Deterministic reproducer for aeron-io/aeron#2213.
 *
 * 1.53.x creates publication images asynchronously: the conductor asks the
 * NativeResourceAgent (NRA) to map the image log buffer and keeps a raw
 * aeron_receive_channel_endpoint_t * in the in-flight driver command.
 *
 * While that command is in flight, the conductor cannot process the receiver's
 * "endpoint removed" notification.  If the last subscription for the endpoint is
 * closed, the endpoint is marked CLOSING and remove_endpoint is sent to the
 * receiver.  When the NRA map completes, the conductor sends a *new* receiver
 * command for the same endpoint (remove_init_in_progress) and only then
 * processes the removal notification.  The endpoint is freed while the new
 * receiver command is still queued.
 *
 * This program:
 *   - starts an embedded C media driver (1.53.3) with ASAN,
 *   - installs an incoming interceptor that blocks the receiver thread once,
 *   - pauses image log buffer mapping via the driver's raw_log_map_func hook,
 *   - subscribes/publishes, then closes the subscription while the image map is
 *     in progress,
 *   - blocks the receiver on a UDP packet until after the endpoint has been
 *     deleted, so the queued command runs on freed memory.
 *
 * Build with ASAN (see README).
 */
#define _GNU_SOURCE

#include <arpa/inet.h>
#include <netinet/in.h>
#include <pthread.h>
#include <sched.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <time.h>
#include <unistd.h>

#include "aeronc.h"
#include "aeronmd.h"
#include "aeron_driver_context.h"
#include "media/aeron_udp_channel_transport_bindings.h"
#include "util/aeron_fileutil.h"

#define STREAM_ID 1001
#define TARGET_CHANNEL "aeron:udp?endpoint=127.0.0.1:33213|term-length=64m"
#define FILLER_CHANNEL "aeron:udp?endpoint=127.0.0.1:34999|term-length=64m"
#define FILLER_PORT 34999

static atomic_bool g_stop = false;
static atomic_bool g_image_map_started = false;
static atomic_bool g_image_map_finished = false;
static atomic_bool g_block_next_packet = false;
static atomic_bool g_receiver_blocked = false;
static atomic_int g_slow_map_ms = 2000;
static atomic_int g_block_ms = 2500;

static void sleep_ms(int ms)
{
    struct timespec ts = { ms / 1000, (long)(ms % 1000) * 1000000L };
    nanosleep(&ts, NULL);
}

static void fail(const char *what)
{
    fprintf(stderr, "[repro] %s failed: %s\n", what, aeron_errmsg());
    exit(EXIT_FAILURE);
}

static void *driver_main(void *arg)
{
    aeron_driver_t *driver = (aeron_driver_t *)arg;
    while (!atomic_load_explicit(&g_stop, memory_order_acquire))
    {
        int work_count = aeron_driver_main_do_work(driver);
        aeron_driver_main_idle_strategy(driver, work_count);
    }
    return NULL;
}

/*
 * raw_log_map_func hook: pause only image log buffers.  This widens the window
 * in which the image creation driver command is in flight.  It does not change
 * driver logic.
 */
static int slow_raw_log_map(
    aeron_mapped_raw_log_t *mapped_raw_log,
    const char *path,
    bool use_sparse_files,
    uint64_t term_length,
    uint64_t page_size)
{
    if (NULL != strstr(path, "/images/"))
    {
        int slow_ms = atomic_load_explicit(&g_slow_map_ms, memory_order_acquire);
        fprintf(stderr, "[repro] image map started: %s (pausing %d ms)\n", path, slow_ms);
        atomic_store_explicit(&g_image_map_started, true, memory_order_release);
        sleep_ms(slow_ms);
        fprintf(stderr, "[repro] image map resuming\n");
    }

    int result = aeron_raw_log_map(mapped_raw_log, path, use_sparse_files, term_length, page_size);
    if (NULL != strstr(path, "/images/"))
    {
        atomic_store_explicit(&g_image_map_finished, true, memory_order_release);
    }
    return result;
}

static int block_init(void **interceptor_state, aeron_driver_context_t *context, aeron_udp_channel_transport_affinity_t affinity)
{
    (void)context;
    (void)affinity;
    *interceptor_state = NULL;
    return 0;
}

static int block_close(void *interceptor_state)
{
    (void)interceptor_state;
    return 0;
}

/*
 * Runs on the receiver thread while it polls a socket.  Blocking here stops the
 * receiver from draining its command queue, so the conductor can free the
 * endpoint before the queued command is handled.  The packet is dropped.
 */
static void block_incoming(
    void *interceptor_state,
    aeron_udp_channel_incoming_interceptor_t *delegate,
    aeron_udp_channel_transport_t *transport,
    void *receiver_clientd,
    void *endpoint_clientd,
    void *destination_clientd,
    uint8_t *buffer,
    size_t length,
    struct sockaddr_storage *addr,
    struct timespec *media_timestamp)
{
    (void)interceptor_state;
    (void)delegate;
    (void)transport;
    (void)receiver_clientd;
    (void)endpoint_clientd;
    (void)destination_clientd;
    (void)buffer;
    (void)length;
    (void)addr;
    (void)media_timestamp;

    if (atomic_exchange_explicit(&g_block_next_packet, false, memory_order_acq_rel))
    {
        int block_ms = atomic_load_explicit(&g_block_ms, memory_order_acquire);
        fprintf(stderr, "[repro] receiver blocked in interceptor for %d ms\n", block_ms);
        atomic_store_explicit(&g_receiver_blocked, true, memory_order_release);
        sleep_ms(block_ms);
        fprintf(stderr, "[repro] receiver unblocked\n");
    }

    if (NULL != delegate && NULL != delegate->incoming_func)
    {
        delegate->incoming_func(
            delegate->interceptor_state,
            delegate->next_interceptor,
            transport,
            receiver_clientd,
            endpoint_clientd,
            destination_clientd,
            buffer,
            length,
            addr,
            media_timestamp);
    }
}

static aeron_udp_channel_interceptor_bindings_t *make_interceptor_bindings(void)
{
    aeron_udp_channel_interceptor_bindings_t *bindings =
        (aeron_udp_channel_interceptor_bindings_t *)calloc(1, sizeof(*bindings));
    if (NULL == bindings)
    {
        return NULL;
    }
    bindings->incoming_init_func = block_init;
    bindings->incoming_close_func = block_close;
    bindings->incoming_func = block_incoming;
    bindings->meta_info.name = "aeron-2213-block";
    bindings->meta_info.type = "incoming";
    bindings->meta_info.next_interceptor_bindings = NULL;
    bindings->meta_info.source_symbol = NULL;
    return bindings;
}

static aeron_subscription_t *add_subscription(aeron_t *aeron, const char *channel, int32_t stream_id)
{
    aeron_async_add_subscription_t *async = NULL;
    aeron_subscription_t *subscription = NULL;

    if (aeron_async_add_subscription(&async, aeron, channel, stream_id, NULL, NULL, NULL, NULL) < 0)
    {
        fail("aeron_async_add_subscription");
    }

    int rc;
    while ((rc = aeron_async_add_subscription_poll(&subscription, async)) == 0)
    {
        sched_yield();
    }
    if (rc < 0)
    {
        fail("aeron_async_add_subscription_poll");
    }
    return subscription;
}

static aeron_publication_t *add_publication(aeron_t *aeron, const char *channel, int32_t stream_id)
{
    aeron_async_add_publication_t *async = NULL;
    aeron_publication_t *publication = NULL;

    if (aeron_async_add_publication(&async, aeron, channel, stream_id) < 0)
    {
        fail("aeron_async_add_publication");
    }

    int rc;
    while ((rc = aeron_async_add_publication_poll(&publication, async)) == 0)
    {
        sched_yield();
    }
    if (rc < 0)
    {
        fail("aeron_async_add_publication_poll");
    }
    return publication;
}

static void send_packet_to_filler(void)
{
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0)
    {
        perror("socket");
        return;
    }

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(FILLER_PORT);
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

    uint8_t payload[64];
    memset(payload, 0, sizeof(payload));
    for (int i = 0; i < 10; i++)
    {
        (void)sendto(fd, payload, sizeof(payload), 0, (struct sockaddr *)&addr, sizeof(addr));
    }
    close(fd);
}

int main(void)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stderr, NULL, _IONBF, 0);

    const char *env_slow = getenv("AERON_2213_SLOW_MS");
    if (NULL != env_slow)
    {
        atomic_store(&g_slow_map_ms, atoi(env_slow));
    }
    const char *env_block = getenv("AERON_2213_BLOCK_MS");
    if (NULL != env_block)
    {
        atomic_store(&g_block_ms, atoi(env_block));
    }
    const int slow_ms = atomic_load(&g_slow_map_ms);
    const int block_ms = atomic_load(&g_block_ms);

    char aeron_dir[512];
    snprintf(aeron_dir, sizeof(aeron_dir), "/tmp/aeron-2213-%d", (int)getpid());

    aeron_driver_context_t *driver_context = NULL;
    if (aeron_driver_context_init(&driver_context) < 0)
    {
        fail("aeron_driver_context_init");
    }
    aeron_driver_context_set_dir(driver_context, aeron_dir);
    aeron_driver_context_set_dir_delete_on_start(driver_context, true);
    aeron_driver_context_set_dir_delete_on_shutdown(driver_context, true);
    aeron_driver_context_set_term_buffer_length(driver_context, 64 * 1024 * 1024);
    aeron_driver_context_set_term_buffer_sparse_file(driver_context, false);
    aeron_driver_context_set_threading_mode(driver_context, AERON_THREADING_MODE_DEDICATED);
    aeron_driver_context_set_timer_interval_ns(driver_context, 1000000);
    aeron_driver_context_set_conductor_idle_strategy(driver_context, "yield");
    aeron_udp_channel_interceptor_bindings_t *interceptor_bindings = make_interceptor_bindings();
    if (NULL == interceptor_bindings)
    {
        fail("make_interceptor_bindings");
    }
    if (aeron_driver_context_set_udp_channel_incoming_interceptors(driver_context, interceptor_bindings) < 0)
    {
        fail("aeron_driver_context_set_udp_channel_incoming_interceptors");
    }
    driver_context->raw_log_map_func = slow_raw_log_map;

    aeron_driver_t *driver = NULL;
    if (aeron_driver_init(&driver, driver_context) < 0)
    {
        fail("aeron_driver_init");
    }
    if (aeron_driver_start(driver, true) < 0)
    {
        fail("aeron_driver_start");
    }

    pthread_t driver_thread;
    if (pthread_create(&driver_thread, NULL, driver_main, driver) != 0)
    {
        fail("pthread_create");
    }
    printf("[repro] driver started in %s (slow map %d ms, block %d ms)\n", aeron_dir, slow_ms, block_ms);

    aeron_context_t *client_context = NULL;
    aeron_t *aeron = NULL;
    if (aeron_context_init(&client_context) < 0)
    {
        fail("aeron_context_init");
    }
    aeron_context_set_dir(client_context, aeron_dir);
    aeron_context_set_idle_strategy(client_context, "yield");
    if (aeron_init(&aeron, client_context) < 0)
    {
        fail("aeron_init");
    }
    if (aeron_start(aeron) < 0)
    {
        fail("aeron_start");
    }

    aeron_subscription_t *target_subscription = add_subscription(aeron, TARGET_CHANNEL, STREAM_ID);
    aeron_subscription_t *filler_subscription = add_subscription(aeron, FILLER_CHANNEL, STREAM_ID + 1);
    printf("[repro] subscriptions added\n");
    sleep_ms(500);

    aeron_publication_t *publication = add_publication(aeron, TARGET_CHANNEL, STREAM_ID);
    printf("[repro] publication added, waiting for image map\n");

    int waited = 0;
    while (!atomic_load_explicit(&g_image_map_started, memory_order_acquire) && waited < 15000)
    {
        sleep_ms(10);
        waited += 10;
    }
    if (!atomic_load_explicit(&g_image_map_started, memory_order_acquire))
    {
        fprintf(stderr, "[repro] image map never started (no SETUP?)\n");
        return EXIT_FAILURE;
    }

    printf("[repro] closing target subscription during image map\n");
    if (aeron_subscription_close(target_subscription, NULL, NULL) < 0)
    {
        fail("aeron_subscription_close");
    }

    /* Let the receiver handle remove_endpoint before we block it. */
    sleep_ms(500);

    printf("[repro] blocking receiver; image map should complete while it is blocked\n");
    atomic_store_explicit(&g_block_next_packet, true, memory_order_release);
    send_packet_to_filler();

    int block_waited = 0;
    while (!atomic_load_explicit(&g_receiver_blocked, memory_order_acquire) && block_waited < 5000)
    {
        sleep_ms(10);
        block_waited += 10;
    }

    /* Wait for the map, the conductor's endpoint deletion, and the receiver wake-up. */
    while (!atomic_load_explicit(&g_image_map_finished, memory_order_acquire))
    {
        sleep_ms(10);
    }
    sleep_ms(block_ms + 3000);

    fprintf(stderr, "[repro] no crash (unexpected)\n");
    /* filler_subscription is only here to create the endpoint that receives the
     * blocking packet; aeron_close() releases it. */
    (void)filler_subscription;
    aeron_publication_close(publication, NULL, NULL);
    aeron_close(aeron);
    aeron_context_close(client_context);
    atomic_store(&g_stop, true);
    pthread_join(driver_thread, NULL);
    aeron_driver_close(driver);
    aeron_driver_context_close(driver_context);
    return EXIT_SUCCESS;
}
