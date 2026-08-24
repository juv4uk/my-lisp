# CPU + GPU + FPGA execution fabric

**Status:** architecture direction, 2026-08-24  
**Scope:** `my-lisp`, CML, and `fpga-lisp`  
**Authority:** machine-readable language, compiler, ABI, and ISA contracts
remain authoritative over this document.

This document is kept in all three repositories because the design crosses
their boundary. It does not merge their authority:

```text
my-lisp    owns observable language semantics
CML        owns analysis, partitioning, planning, and lowering
fpga-lisp  owns the FPGA ISA, RTL, transport, and hardware evidence
```

## Goal

Make the CPU, a portable or vendor-specific GPU, and the FPGA cooperate on
one program without making any device part of the language semantics.

```text
                     my-lisp program
                            |
                    canonical semantics
                            |
                      CML semantic IR
                            |
               analysis + Execution Graph
                 +----------+----------+
                 |          |          |
                CPU        GPU        FPGA
             full Lisp   bulk data   streams /
              runtime    parallelism  pipelines
```

The source says what to compute. CML decides where a proven-safe region may
run. A backend performs the physical work. Moving a region must not change
its observable value or error.

## Complementary roles

### CPU: coordinator and semantic fallback

The CPU runs the complete canonical semantics: dynamic control, closures,
environments, macros after expansion, exact bignums and rationals, allocation,
GC, host capabilities, errors, and irregular recursion. It also owns the
scheduler and is the mandatory fallback when an accelerator is unavailable or
ineligible.

### GPU: portable bulk compute

The GPU handles pure operations over immutable typed numeric buffers:
element-wise maps, zips, reductions, and later dense numeric kernels. The
first portable implementation should be Rust plus `wgpu`; a CUDA backend may
remain as an NVIDIA-specific optimized path. Future Intel and AMD paths enter
through the same capability contract, not through new Lisp syntax.

```text
Compute IR
  +-- cpu-reference
  +-- gpu-wgpu  -> Vulkan / DX12 / Metal / WebGPU
  +-- gpu-cuda  -> NVIDIA-specific path
  +-- gpu-level-zero / oneAPI (planned capability)
  `-- gpu-rocm (planned capability)
```

Only a capability proven live on the current machine may be selected.
`planned` is documentation, never executable availability. Exact values are
never silently converted to `f32`; `i32` overflow and `f32` rounding obey the
ratified numeric-buffer contract.

### FPGA: deterministic machine and pipeline executor

The FPGA has two distinct roles:

1. execute general Lisp lowered to the `fpga-lisp` ISA;
2. eventually execute specialized stream/dataflow regions as hardware
   pipelines.

The first role is already physically evidenced by SRAM programming and UART
execution on the connected GW5A-25A board. The second remains a planned
specialization and must not be reported as implemented merely because the
general Lisp machine works.

## Execution Graph

CML needs a device-neutral graph above concrete backends. A minimal model is:

```rust
enum ExecutionTarget {
    Cpu,
    Gpu { backend: GpuBackend },
    Fpga { device: String },
}

struct PlanNode {
    id: NodeId,
    operation: Operation,
    inputs: Vec<BufferId>,
    outputs: Vec<BufferId>,
    dependencies: Vec<NodeId>,
    target: ExecutionTarget,
}
```

The graph describes dependencies, values crossing device boundaries, and the
selected executor. It must not contain raw host or device pointers. Backends
exchange immutable logical buffers through explicit handles and transfers:

```text
CPU buffer --upload--> GPU buffer --kernel--> GPU buffer --download--> CPU
CPU buffer --frame---> FPGA UART/PCIe/other transport --result frame--> CPU
```

M0 may materialize every boundary through host memory. Zero-copy, pinned
memory, shared virtual memory, and direct device-to-device transport are later
optimizations and may be used only when they preserve the same graph contract.

## Backend contract

Every executor reports capabilities and uses the same lifecycle:

```rust
trait ExecutionBackend {
    fn capabilities(&self) -> BackendCapabilities;
    fn prepare(&mut self, node: &PlanNode, buffers: &BufferStore)
        -> Result<PreparedJob, BackendError>;
    fn execute(&mut self, job: PreparedJob) -> Result<JobResult, BackendError>;
}
```

Preparation performs all checks possible before side effects. Execution
publishes outputs atomically: failure never exposes a partial language value.
Backend errors are named and distinguish unsupported operation, unavailable
device, transfer failure, device loss, arithmetic failure, and timeout.

## Planner rules

A target is eligible only when all required facts are known:

```text
semantic equivalence obligation satisfied
+ operation/effect shape supported
+ value representation supported
+ device capability status = live
+ transfer and launch protocol available
= eligible target
```

Unknown facts reject acceleration. Initial planning is explicit and
deterministic. Automatic selection comes only after differential correctness
and measurements establish useful thresholds.

Suggested execution shapes:

| Shape | Primary candidate | Reason |
|---|---|---|
| dynamic, recursive, stateful | CPU | full semantics and irregular control |
| pure element-wise / reduction | GPU | massive fixed-width parallelism |
| deterministic stream / pipeline | FPGA | spatial dataflow and stable latency |
| unsupported or uncertain | CPU | canonical fallback |

The planner considers semantics first, then availability, transfer cost,
problem size, arithmetic intensity, and queue load. Vendor identity is a
backend detail.

## Conformance and evidence

Each specialized result is compared with the canonical path:

```text
my-lisp evaluator / CPU reference -> expected
CML selected backend              -> actual
expected == actual under the operation's declared numeric contract
```

For `i32`, equality includes identical values and identical overflow behavior.
For deterministic `f32` operations, compare bits; where a later operation
permits backend variation, name the tolerance in the contract rather than in
an ad-hoc test. FPGA claims distinguish simulation, synthesized bitstream,
SRAM programming, UART transport, and observed hardware result.

The conformance matrix uses explicit states:

```text
CONFIRMED | PARTIAL | UNSUPPORTED | UNAVAILABLE | BROKEN | UNRESOLVED
```

No row is inferred from another backend's success.

## Incremental milestones

### M0 — graph before acceleration

- add the Execution Graph and logical buffer handles to CML;
- execute a multi-node graph entirely through the CPU reference backend;
- validate dependency ordering, buffer ownership, failure propagation, and
  deterministic results;
- represent GPU and FPGA targets but reject them unless a live executor is
  registered.

### M1 — CPU + GPU

- lower canonical `numeric-buffer-map` into the graph;
- run the same graph through CPU and one live GPU backend;
- keep `wgpu` as the portable direction and CUDA as an optional optimized
  NVIDIA path;
- record transfer and kernel timing without yet making automatic placement a
  semantic promise.

### M2 — CPU + FPGA

- define a versioned CML/fpga-lisp job and result frame;
- replace script-only orchestration with a Rust transport implementation while
  retaining the existing monitor as an independent diagnostic tool;
- upload and execute a graph node on the physical board;
- report transport, program, device identity, and result evidence separately.

### M3 — one heterogeneous golden experiment

Use a deliberately small, inspectable pipeline:

```text
CPU: create and validate input
 -> GPU: pure numeric-buffer map
 -> CPU: convert/partition results
 -> FPGA: deterministic Lisp or stream computation
 -> CPU: validate final result against the reference execution
```

This milestone proves orchestration, not universal speedup. Performance claims
require later workloads and measurements.

### M4 — cost-aware placement

- collect per-device transfer, launch, and execution measurements;
- choose targets only when the estimated benefit exceeds the movement cost;
- preserve an explainable plan showing why each node was placed;
- add fallback/retry policy without executing a non-idempotent node twice.

## Explicit non-goals

- No `cuda-*`, `intel-*`, `amd-*`, or `fpga-*` primitives in core my-lisp.
- No implicit exact-to-inexact conversion.
- No requirement that every backend implement full Lisp.
- No shared raw pointers as a cross-device ABI.
- No automatic planner before correctness and cost evidence.
- No claim that FPGA dataflow specialization exists before RTL and hardware
  evidence.

## First implementation decision

The next code slice belongs in CML: implement M0's Execution Graph with the
CPU reference executor and fail-closed target admission. This creates the
stable seam to which the already-live CUDA path, a portable `wgpu` path, and
the physically connected FPGA can attach without coupling any vendor or
transport to language semantics.
