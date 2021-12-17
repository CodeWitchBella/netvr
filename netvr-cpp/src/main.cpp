#include "netvr-internal.h"
#include "../vendor/openxr/openxr.h"
#include <cstdint>

using std::uint64_t;

void (*log)(const char *);

ISBL_NETVR_EXPORT int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr)
{
    log("On system change");
    return 22;
}

ISBL_NETVR_EXPORT void isbl_netvr_set_logger(void (*callback)(const char *))
{
    log = callback;
}
