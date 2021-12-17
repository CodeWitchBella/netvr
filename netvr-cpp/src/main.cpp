#include "netvr-internal.h"
#include "../vendor/openxr/openxr.h"
#include <cstdint>

using std::uint64_t;

void (*log)(const char *);

PFN_xrGetInstanceProcAddr orig_get_instance_proc_addr;

XrResult my_get_instance_proc_addr(XrInstance instance, const char *name, PFN_xrVoidFunction *function)
{
    log(name);
    return orig_get_instance_proc_addr(instance, name, function);
}

ISBL_NETVR_EXPORT void *isbl_netvr_hook_get_instance_proc_addr(void *func)
{
    log("isbl_netvr_hook_get_instance_proc_addr");
    orig_get_instance_proc_addr = (PFN_xrGetInstanceProcAddr)func;
    return (void *)my_get_instance_proc_addr;
}

ISBL_NETVR_EXPORT int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr)
{
    log("On system change");
    return 22;
}

ISBL_NETVR_EXPORT void isbl_netvr_set_logger(void (*callback)(const char *))
{
    log = callback;
}
