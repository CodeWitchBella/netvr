#include "netvr-internal.h"
#include "../vendor/openxr/openxr.h"
#include <cstdint>

#include <Eigen/Dense>

using std::uint64_t;

void (*unity_log)(const char *);

PFN_xrGetInstanceProcAddr orig_get_instance_proc_addr;

/**
 * This is the function that is returned by the hook and as such is the function
 * that is called by the engine when it wants to get a function pointer to an
 * OpenXR function.
 *
 * @param instance
 * @param name
 * @param function
 * @return XrResult
 */
XrResult my_get_instance_proc_addr(XrInstance instance, const char *name, PFN_xrVoidFunction *function)
{
    unity_log(name);
    return orig_get_instance_proc_addr(instance, name, function);
}

// see netvr.h for docs
ISBL_NETVR_FUNC void *isbl_netvr_hook_get_instance_proc_addr(void *func)
{
    unity_log("isbl_netvr_hook_get_instance_proc_addr");
    orig_get_instance_proc_addr = (PFN_xrGetInstanceProcAddr)func;
    return (void *)my_get_instance_proc_addr;
}

// see netvr.h for docs
ISBL_NETVR_FUNC int isbl_netvr_on_system_change(uint64_t xrSystem, uint64_t xrInstance, void *xrGetInstanceProcAddrPtr)
{
    unity_log("On system change");

    using Eigen::MatrixXd;
    MatrixXd m(2, 2);
    m.setIdentity();
    return m.determinant();
}

// see netvr.h for docs
ISBL_NETVR_FUNC void isbl_netvr_set_logger(void (*callback)(const char *))
{
    unity_log = callback;
}
