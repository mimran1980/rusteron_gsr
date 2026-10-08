#include "media/aeron_udp_channel_transport_bindings.h"

/* The size of the bindings table a custom UDP transport fills in. */
size_t links_test_transport_bindings_size(void)
{
    return sizeof(aeron_udp_channel_transport_bindings_t);
}
