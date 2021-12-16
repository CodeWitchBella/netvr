// includes/netvr.h is meant to be as useable in programs outside of netvr-cpp
// (for example future unreal engine integration)
// that means that it tries to not polute global namespace and also that it
// defaults to signatures meant for *importing* dlls. Following two defines
// switch that behaviour around.
#define WIN_EXPORT
#define ISBL_NETVR_EXPORT_NO_UNDEF

#include "../includes/netvr.h"
