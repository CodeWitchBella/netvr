#include "netvr-internal.h"
#include "../vendor/openxr/openxr.h"
#include <cstdint>

using std::uint64_t;

ISBL_NETVR_EXPORT int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr)
{
    return 22;
}
