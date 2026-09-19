#include "wsm_kernel.h"

static WsmStatus noop_start(void *ctx) { (void)ctx; return WSM_STATUS_OK; }
static WsmStatus noop_stop(void *ctx) { (void)ctx; return WSM_STATUS_OK; }
static WsmStatus noop_exchange(void *ctx, WsmByteSpan req, WsmMutableByteSpan out, size_t *written) {
    (void)ctx; (void)req; (void)out;
    if (written) *written = 0;
    return WSM_STATUS_OK;
}
static WsmStatus noop_snapshot(void *ctx, WsmMutableByteSpan out, size_t *written) {
    (void)ctx; (void)out;
    if (written) *written = 0;
    return WSM_STATUS_OK;
}

int main(void) {
    WsmKernelVTable kernels[4] = {
        {WSM_KERNEL_ABI_VERSION, WSM_KERNEL_LISP, 0, noop_start, noop_exchange, noop_snapshot, noop_stop},
        {WSM_KERNEL_ABI_VERSION, WSM_KERNEL_PROLOG, 0, noop_start, noop_exchange, noop_snapshot, noop_stop},
        {WSM_KERNEL_ABI_VERSION, WSM_KERNEL_CLIPS, 0, noop_start, noop_exchange, noop_snapshot, noop_stop},
        {WSM_KERNEL_ABI_VERSION, WSM_KERNEL_DATALOG, 0, noop_start, noop_exchange, noop_snapshot, noop_stop}
    };
    return kernels[0].kernel == WSM_KERNEL_LISP && kernels[3].kernel == WSM_KERNEL_DATALOG ? 0 : 1;
}
