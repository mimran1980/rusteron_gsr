# Reproducer for aeron-io/aeron#2213

**Use-after-free of `aeron_receive_channel_endpoint_t` during asynchronous publication image creation.**

This directory contains a small, deterministic reproducer for
[aeron-io/aeron#2213](https://github.com/aeron-io/aeron/issues/2213). It builds
Aeron **1.53.3** from the upstream tag with AddressSanitizer and starts an
embedded C media driver. It is a test harness; it is not intended for
production.

The reproducer is deterministic (3/3 runs on 1.53.3) and produces:

```
ERROR: AddressSanitizer: heap-use-after-free
READ of size 8
  #0 aeron_int64_to_ptr_hash_map_get
  #1 aeron_data_packet_dispatcher_remove_matching_state
  #2 aeron_receive_channel_endpoint_on_remove_matching_state
  #3 aeron_driver_receiver_on_remove_matching_state
  ...
freed by thread T4 here:
  #2 aeron_receive_channel_endpoint_delete
  #3 aeron_receive_channel_endpoint_entry_delete
  #4 aeron_driver_conductor_on_check_managed_resources
  ...
```

Aeron **1.51.0** (the last good version in the issue) does **not** reproduce
it. The regression was introduced when log-buffer creation was moved off the
conductor thread in **1.52.0**.

---

## 1. Environment

| Item | Value |
|---|---|
| Aeron C driver | **1.53.3** (upstream tag) |
| Last good | 1.51.0 |
| `term_buffer_sparse_file` | `false` |
| Driver mode | `DEDICATED` |
| OS | Linux x86_64 (tested under Docker amd64 emulation on arm64 macOS; native x86_64 is fine) |
| Detector | AddressSanitizer (`-DSANITISE_BUILD=ON` + `-fsanitize=address,undefined`) |

Aeron 1.53.3's `CMakeLists.txt` requires CMake >= 3.30. Ubuntu 24.04 ships
3.28, so the Dockerfile installs CMake 3.30.9. This is a build-tool detail and
does not change any C source under test.

## 2. Run it

```bash
docker build --platform linux/amd64 -t aeron-2213-repro .
docker run --rm --platform linux/amd64 aeron-2213-repro
```

On a native x86_64 Linux host the `--platform` flags can be omitted (or you can
use the native build commands in section 8).

The process aborts with the ASAN report. The full expected report is in
[`expected-asan.txt`](expected-asan.txt).

### Tuning

The two hooks are controlled by environment variables:

| Variable | Default | Meaning |
|---|---|---|
| `AERON_2213_SLOW_MS` | `2000` | How long the image log-buffer map is paused. |
| `AERON_2213_BLOCK_MS` | `2500` | How long the receiver thread is blocked. |

If it ever does not abort (for example on a very different scheduler), increase
`AERON_2213_SLOW_MS` or `AERON_2213_BLOCK_MS`.

## 3. What the program does

1. Starts an embedded media driver (`aeron_driver_init` / `aeron_driver_start`).
2. Installs two **test hooks** (they only widen the race, they do not change
   driver logic):
   * `raw_log_map_func` pauses mappings whose path contains `/images/`
     (`AERON_2213_SLOW_MS`).
   * An incoming UDP interceptor blocks the **receiver thread** once on a
     packet sent to the filler endpoint (`AERON_2213_BLOCK_MS`). While blocked,
     the receiver cannot drain its command queue.
3. Creates a target subscription (`aeron:udp?endpoint=127.0.0.1:33213|term-length=64m`,
   stream `1001`) and a filler subscription (`127.0.0.1:34999`, stream `1002`).
   The filler exists only so there is a live endpoint to send the blocking
   packet to.
4. Adds a publication on the target channel. The receiver sees the publisher's
   `SETUP` and asks the conductor to create a publication image. The conductor
   schedules an asynchronous driver command and asks the NRA to map the image
   log buffer. The hook pauses inside that map.
5. While the map is paused, closes the target subscription. Its receive
   endpoint is marked `CLOSING` and `remove_endpoint` is sent to the receiver.
   The receiver processes `remove_endpoint` and notifies the conductor, but the
   conductor is still inside the in-flight image-creation driver command, so the
   notification waits.
6. Blocks the receiver on the filler packet. The map then completes.
7. The conductor's `create_image` step sees the endpoint in `CLOSING` state and
   queues `on_remove_init_in_progress` to the receiver **after**
   `remove_endpoint`. The conductor then processes the delayed
   `on_receive_endpoint_removed`, marks the endpoint receiver-released, and the
   end-of-life check frees it.
8. The receiver wakes and processes `on_remove_init_in_progress`, which
   dereferences the freed endpoint/dispatcher: ASAN reports
   `heap-use-after-free`.

Without the two hooks, the same window exists naturally: `term-length=64m` with
`term_buffer_sparse_file=false` touches ~192 MiB of pages during the NRA map,
and a mass client restart queues many receiver commands behind the delayed
removal notification.

## 4. Root cause

In 1.52.0 log-buffer creation/removal was moved from the conductor to the
`NativeResourceAgent` (see the 1.52.0 changelog: *"Move log buffer creation/removal
out of the conductor thread"*).

`aeron_driver_conductor_on_create_publication_image()`
(`aeron-driver/src/main/c/aeron_driver_conductor.c:6848` in 1.53.3) copies the
command, which contains raw `endpoint` and `destination` pointers, into an
asynchronous driver command. No reference is taken on the endpoint.

The command is executed in three states:

* `aeron_driver_conductor_execute_create_publication_image_validate()`
  (`.../aeron_driver_conductor.c:6490`)
* `aeron_driver_conductor_execute_create_publication_image_await_log_buffer()`
  (`.../aeron_driver_conductor.c:6565`)
* `aeron_driver_conductor_execute_create_publication_image_create_image()`
  (`.../aeron_driver_conductor.c:6591`)

While this command returns `RUNNING` (waiting for the NRA),
`aeron_driver_conductor_process_driver_commands()`
(`.../aeron_driver_conductor.c:3339`) does **not** read new commands from the
receiver-proxy queue, because `conductor->driver_command` is non-NULL. This
includes `aeron_driver_conductor_on_receive_endpoint_removed()`
(`.../aeron_driver_conductor.c:7040`).

The endpoint lifetime code is in
`aeron-driver/src/main/c/media/aeron_receive_channel_endpoint.c`:

* `aeron_receive_channel_endpoint_try_remove_endpoint()` (line 691) marks the
  endpoint `CLOSING` when all stream/session/response maps are empty and
  `image_ref_count <= 0`, then sends `remove_endpoint` to the receiver.
* `image_ref_count` is incremented only **after**
  `aeron_publication_image_create()` succeeds (conductor line 6733-6741), i.e.
  after the NRA map completes. It is still 0 while the image-creation command is
  in flight.
* `aeron_receive_channel_endpoint_entry_delete()` (conductor line 1564) frees
  the endpoint from the conductor's end-of-life check once
  `has_receiver_released` is true.

Sequence that produces the UAF:

1. Target subscription closes: `try_remove_endpoint` sees all refs zero, sets
   `CLOSING`, sends `remove_endpoint`.
2. Receiver removes the transports, notifies the conductor, and (in the
   reproducer) is blocked.
3. The NRA map completes. `create_image` sees `CLOSING`, so it queues
   `on_remove_init_in_progress` for the same endpoint to the receiver. That
   command is now behind `remove_endpoint` in the receiver's queue.
4. The conductor finally processes the delayed removal notification, sets
   `has_receiver_released`, and the end-of-life check calls
   `aeron_receive_channel_endpoint_delete()`, which frees the endpoint, its
   components and its dispatcher.
5. The receiver reaches `on_remove_init_in_progress` and reads the freed
   dispatcher/endpoint.

`aeron_receive_channel_endpoint_try_remove_endpoint()` also does not check
whether the endpoint is already `CLOSING`; a second call can enqueue a second
`remove_endpoint`, which is another way for the receiver to touch a freed
endpoint (the `remove_poll_transports` crash in the issue).

## 5. Why 1.51.0 does not crash

In 1.51.0 `aeron_publication_image_create()` (including the raw-log map) runs
**synchronously on the conductor thread**. There is no in-flight async command,
so the conductor cannot mark the endpoint `CLOSING` and delete it between the
status check and image creation. The endpoint is safe for the duration of the
call. This is why the reproducer prints `no crash` against 1.51.0.

## 6. Relation to the crashes in the issue

The issue reports four different faulting functions. They are all consistent
with an endpoint being freed while a command still references it:

| Issue crash | What touches the freed endpoint |
|---|---|
| `aeron_receive_channel_endpoint_remove_poll_transports` (receiver) | A queued/second `remove_endpoint` command. |
| `aeron_receive_channel_endpoint_decref_to_stream` (conductor) | A `network_subscriptions` link whose endpoint was already freed. |
| `aeron_driver_conductor_execute_add_network_subscription` (conductor) | Same lifetime problem on a new subscription. |
| `aeron_data_packet_dispatcher_remove_publication_image` (receiver) | A receiver command queued after `remove_endpoint` (the image-teardown variant of this reproducer). |

This reproducer demonstrates the concrete receiver-side path
(`remove_init_in_progress` after `remove_endpoint`). A fix that pins the
endpoint for the whole async image-creation command (and makes the receiver's
final free ordered) covers this path; the conductor-side sites should be
verified against the same fix.

## 7. Suggested fix direction

The conductor is the owner of endpoint lifetime, so the async command must not
outlive the endpoint's reference:

1. **Pin the endpoint (and destination) for the async image-creation command.**
   Add a reference (a new `command_ref_count`, or carefully use
   `image_ref_count`) when the command is scheduled, and release it in
   `aeron_driver_conductor_free_create_publication_image()` on every path
   (success, error, cancellation, driver close). `try_remove_endpoint` must
   consider this reference so it cannot mark/free the endpoint while the
   command is in flight. After releasing, call
   `aeron_receive_channel_endpoint_try_remove_endpoint()` so an endpoint whose
   last subscription went away during the map can still be closed.
2. **Do not free the endpoint from the conductor while the receiver may still
   have commands for it.** Either let the receiver perform the final free after
   it has drained all commands up to `remove_endpoint`, or add a sequence
   barrier/ack so the conductor knows the receiver has passed the point after
   which no new endpoint commands can be queued.
3. **Make `aeron_receive_channel_endpoint_try_remove_endpoint()` idempotent**
   by returning early unless the status is `ACTIVE`. This prevents duplicate
   `remove_endpoint` commands (the `remove_poll_transports` crash).
4. As a defence in depth, in
   `aeron_driver_conductor_execute_create_publication_image_create_image()`, if
   the endpoint is not `ACTIVE`, free the mapped log buffer and avoid touching
   the endpoint/receiver. This alone is not sufficient: the status read itself
   is a use-after-free if the endpoint has already been freed.

## 8. Native x86_64 build (without Docker)

```bash
git clone --depth 1 --branch 1.53.3 https://github.com/aeron-io/aeron.git
cd aeron

cmake -S . -B build \
  -DCMAKE_BUILD_TYPE=Debug \
  -DBUILD_AERON_DRIVER=ON \
  -DBUILD_AERON_ARCHIVE_API=OFF \
  -DAERON_TESTS=OFF -DAERON_UNIT_TESTS=OFF -DAERON_SYSTEM_TESTS=OFF \
  -DAERON_BUILD_SAMPLES=OFF -DAERON_BUILD_DOCUMENTATION=OFF \
  -DAERON_INSTALL_TARGETS=OFF \
  -DSANITISE_BUILD=ON
cmake --build build --target aeron aeron_driver -j"$(nproc)"

gcc -g -O1 -fno-omit-frame-pointer -fsanitize=address,undefined \
  -I aeron-client/src/main/c -I aeron-driver/src/main/c \
  /path/to/repro.c -o /tmp/repro \
  -L build/lib -laeron_driver -laeron -lpthread -lbsd -lrt -lm -luuid

LD_LIBRARY_PATH=build/lib \
ASAN_OPTIONS=verify_asan_link_order=0:detect_leaks=0:abort_on_error=1 \
  /tmp/repro
```

## 9. Rusteron

`rusteron-media-driver` 0.2.13 embeds this exact C driver (Aeron 1.53.3), which
is why the issue's stack contains
`rusteron_media_driver::AeronDriver::main_do_work`. The bug is in the C driver,
not in the Rust bindings. The C reproducer above is the smallest artifact for
Aeron upstream; a rusteron harness can follow the same sequence (embedded
driver, add subscription/publication, close the subscription while the image
log buffer is being mapped). The two hooks are only used to make the race
deterministic and are not required by the production trigger.

## 10. Notes

* The reproducer uses internal headers (`aeron_driver_context.h`,
  `media/aeron_udp_channel_transport_bindings.h`) only for the two test hooks.
  The driver logic under test is the unmodified 1.53.3 source.
* It is intentionally single-process and uses loopback UDP; no external media
  driver or network is required.
* On shutdown without a crash the driver directory is removed
  (`dir_delete_on_start`/`dir_delete_on_shutdown` are enabled).
