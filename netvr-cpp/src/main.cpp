#define ISBL_NETVR_EXPORT_NO_UNDEF
#include "../includes/netvr.h"
#include <cstdint>

using std::uint64_t;

ISBL_NETVR_EXPORT int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr)
{
    return 17;
}
