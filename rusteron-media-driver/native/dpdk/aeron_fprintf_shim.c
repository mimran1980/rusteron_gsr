/*
 * rusteron-media-driver DPDK transport — minimal aeron_fprintf shim.
 *
 * transport.c uses AERON_SET_ERR, which routes through the client util's
 * aeron_err_set(); that unit references aeron_fprintf (via the AERON_FPRINTF
 * macro) which the shared libaeron_driver.so does not provide and the client
 * libaeron.so is dropped from the link (--as-needed, its only requester being
 * this archive). Rather than pull the whole client into the media driver,
 * provide the default-handler behaviour here.
 *
 * This matches aeronc.c's aeron_default_fprintf: a plain vfprintf to the given
 * stream. A custom handler installed via aeron_set_fprintf_handler is not
 * consulted; the DPDK transport never installs one.
 */
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>

int aeron_fprintf(const char *src_, uint64_t line_, void *stream, const char *format, ...)
{
    (void)src_;
    (void)line_;
    va_list list;
    va_start(list, format);
    int ret = vfprintf((FILE *)stream, format, list);
    va_end(list);
    return ret;
}
