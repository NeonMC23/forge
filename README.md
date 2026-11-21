# forge workspace

Shared development workspace for the RAMforge and forgeCORE projects.

```text
RAMforge/   RAMforge runtime, planner, accounting, CLI/TUI
forgeCORE/  safe Rust bindings and integration layer over llama.cpp/ggml
native/     external llama.cpp/ggml checkout and build (not tracked)
```

`RAMforge → forgeCORE → llama.cpp/ggml` is the dependency direction. RAMforge
must not call native llama.cpp/ggml symbols directly or depend on `forge-sys`.

The three Git repositories are independent:

- workspace root → `NeonMC23/forge` (this snapshot)
- `RAMforge/` → `NeonMC23/RAMforge`
- `forgeCORE/` → `NeonMC23/forgeCORE`

Nested `.git/`, native source/build trees, models, caches, and build outputs
are excluded from this snapshot. See [WORKFLOW.md](WORKFLOW.md) for the local
engineering and publication policy.

## Native baseline

- llama.cpp/ggml `v0.5.0`
- commit `7fe450e19305b828c199d602c23a8337aaa1f03b`
- shared libraries: `native/build/bin`
- `FORGE_LLAMA_DIR` selects the native tree; see
  [forgeCORE/docs/NATIVE.md](forgeCORE/docs/NATIVE.md).

## Current architecture coverage (2026-10-25)

| Architecture | Legacy RAMforge executor | forgeCORE/libllama | Evidence |
|---|---|---|---|
| `llama` | yes | yes | SmolLM2-135M Q4_K_M |
| `qwen2` | yes | yes | Qwen2.5-1.5B Q4_0 |
| `gptneox` / `gpt_neox` | not-yet (explicit, actionable rejection) | yes | Pythia-70M-deduped Q8_0; native and independent NumPy parity; GPT-NeoX-20B Q4_K_M under an 8 GiB cgroup cap |
| `qwen3`, `qwen35`, `gemma`, `phi`, literal `mistral` | not-yet | not-yet | inspect/plan only |

`gptneox` execution must be requested explicitly with `--backend forgecore`; the
legacy path never falls back or approximates it. The 20B GPT-NeoX GGUF
(13,135,032,320 B file, 13,133,230,080 tensor bytes, SHA-256
`1a35c13535c26bb925a2379d9f021de67a0390124ffe2051aa94412002652d6d`) executed
under a hard 8 GiB cgroup v2 cap with `--enforce-ram --no-repack --no-prefetch`:
7.64 GiB of the 12.23 GiB mapping resident, 7,979 limit events, no OOM kill,
one greedy token ` Paris` in 7.72 s. The same run with prefetching took 16.94 s.
The opt-in no-prefetch mode avoids `MAP_POPULATE` via a documented native
patch; it is still OS demand paging, not a RAMforge per-layer staging engine.
The same patch adds model-scoped `llama_model_release_pages()` and RAMforge
`--release-mapping-pages`; on the 20B prefetch run it released 7.93 GiB to 0,
and decode still produced ` Paris`. It also adds exact tensor-to-mapping
metadata and `--inspect-layer`/`--release-layer`: Qwen2.5-1.5B layer 0
released 25.12 MiB with shared pages skipped, and the 20B layer 0 submitted
293.93 MiB but showed no RSS drop because those pages were already
nonresident under the 8 GiB cap. A two-model fixture test released only the
target mapping (`A 8192->0, B 8192->8192`).

RAMforge now has a graph-aware residency engine:
`--residency-policy off|observe|conservative|memory-bound|reuse-aware|active`,
a unified model-scoped memory ledger with provenance and reconciliation,
in-memory execution profiles, deterministic planning, admission, bounded
deduplicated staging, versioned JSON operation reports, and lifecycle guards.
It releases only after the context is dropped at a safe boundary; it has no
intra-decode eviction. Qwen2.5-1.5B at 1 GiB with active target 512M submitted
514.44 MiB and observed mapping RSS `855.68 -> 476.57 MiB` and process RSS
`1021.21 -> 642.10 MiB`; the soft target was still not met (observed 642.10
MiB), and cgroup peak remained 1 GiB. Observe mode performed no mutation.

The engine also has a native execution-quiescence contract. A context holds a
model-scoped shared lease from asynchronous graph submission until native
synchronization/destruction; page release takes a non-blocking exclusive
attempt and returns `Busy` while any context is in flight. forgeCORE exposes
model/context identity, execution generations, quiescence, and typed residency
errors. RAMforge binds those facts into its planner and resource-lifetime state
machine, and only marks resources release-eligible after the operation boundary
is observed quiescent. This is still between-operation physical page control,
not intra-decode eviction.

RAMforge also exposes an opt-in CPU node-dispatch observer through forgeCORE.
It emits one event from worker 0 before each non-empty, compute-enabled node,
with model/context/execution identity, node/op/source names, bounded storage,
and drop accounting. Native tensor metadata now includes model-generation-
scoped mapping identity and validated page ranges, and opt-in bounded Linux
`mincore()` sampling can run before dispatch. Dispatch and sampling are
observations only: not a tensor read, page fault, I/O, completion, residency
during execution, or release authorization. See
[RAMforge/docs/handoff/GRAPH_SAFETY.md](RAMforge/docs/handoff/GRAPH_SAFETY.md).

RAMforge also has a deterministic predictive staging layer over validated
profiles. Predictions are bounded by window/speculative budgets, admission, and
confidence thresholds; they never authorize release. Feedback history is
validated and atomic, and `ramforge simulate` provides a deterministic
simulation-only comparison. A five-arm, four-workload benchmark harness with
rotated order, page-fault/RSS/cgroup reporting, and an isolated-copy
controlled-cache mode found no mapping/RSS reduction and no universal speedup;
end-to-end timing includes prediction/staging/setup, and a 256 MiB cgroup run
showed lower TTFT but worse end-to-end latency on short workloads. Linux
`mincore()` observation now samples selected prefetch resources first, reports
partial selected/general coverage separately, and labels unknown pages. A safe
`POSIX_FADV_DONTNEED` experiment is applied only to an isolated model copy and
verified with `mincore`; warm, controlled, and preparation-failure states stay
distinct. Before-access residency remains unavailable, so eviction-before-use
and causal benefit stay unknown.
See [RAMforge/docs/handoff/PREDICTION.md](RAMforge/docs/handoff/PREDICTION.md)
and [RAMforge/docs/handoff/BENCHMARKS.md](RAMforge/docs/handoff/BENCHMARKS.md).

It also has an opt-in bounded scheduler graph trace and manifest,
bounded `MADV_WILLNEED` staging, and a capability report. Qwen trace with a
4096-event budget retained 958 requested/958 synchronized completions,
958 nodes/1,050 resources, known graph edges, and 1,050 graph-order last uses;
`intra_decode_safe=false`. A 128M staging request selected 95 tensors and
submitted 127.72 MiB, but mapping RSS remained `0 -> 0`; prefetch is a hint,
not residency. On SmolLM2-135M Q4_K_M at a 128 MiB soft target (4 greedy
tokens, CPU, 2 runs), memory-bound submitted 38,027,264 B and observed mapping
RSS `-38,027,264 B` and process RSS about `163.8 -> 127.5 MiB`; reuse-aware
used a complete 16,384-event profile, submitted 46,702,592 B, and observed
`171.8 -> 127.3 MiB`. Cgroup current did not move because it is whole-cgroup
accounting. Conservative selected nothing because every aligned tensor range
had shared page bytes. `conservative`, `memory-bound`, `reuse-aware`, JSON reports, bounded/validated
profile persistence (`--residency-profile-load`/`--residency-profile-save` and
`ramforge profile`), and the benchmark harness are documented in
[RAMforge/docs/handoff/MEMORY_ACCOUNTING.md](RAMforge/docs/handoff/MEMORY_ACCOUNTING.md),
[RAMforge/docs/handoff/GRAPH_SAFETY.md](RAMforge/docs/handoff/GRAPH_SAFETY.md),
and [RAMforge/docs/handoff/BENCHMARKS.md](RAMforge/docs/handoff/BENCHMARKS.md).
The 4G/1G/100M 20B cases were not run. See
[RAMforge/README.md](RAMforge/README.md#enforced-host-memory-budgets-cgroup-v2)
for the enforced matrix, performance, and limitations.

### Architecture-aware planning and release throughput (2026-11-21)

RAMforge now records dense/MoE architecture metadata from GGUF (layers, heads,
KV heads, context, expert counts, and a routed-expert fraction that is not a
complete active-parameter ratio). A versioned memory hierarchy separates host
RAM, cgroup budget, per-device memory, storage, file-backed mappings,
anonymous/process memory, KV estimate, compute/graph, backend buffers, and
staging; overlapping categories are never summed, and placement stays unknown
unless evidence exists. The local service exposes `architecture` and
`hierarchy` in addition to `capabilities` and `status`.

Release-build throughput on SmolLM2-135M Q4_K_M (256 context, 32 tokens, 5
runs + 1 warmup, CPU-only, warm/uncontrolled): balanced 8 threads 283.81 tok/s
(P25–P75 281.77–285.24), balanced 4 threads 267.65 (259.09–267.99),
maximum-performance 4 threads repacked 263.47, background 102.80. An
order-reversed session reproduced the 8-over-4 ordering; absolute rates vary
between sessions. Benchmark JSON now carries the resolved plan and exact
effective configuration.

A fresh 20B out-of-core run re-verified the existing paging path: 13.13 GB
GPT-NeoX-20B Q4_K_M under an 8 GiB cgroup with `--no-repack --no-prefetch`
produced one token, exit 0, 10.877 s wall, 9661 `max` events, and 0 OOM kills.
This is fit under reclaim, not throughput, and page release/prefetch remains OS
paging rather than tensor/layer streaming. No GPU backend was built or
validated. See
[RAMforge/docs/handoff/LARGE_MODEL_EXECUTION.md](RAMforge/docs/handoff/LARGE_MODEL_EXECUTION.md).

### CUDA execution validated (2026-11-21)

The reference RTX 3050 Laptop GPU (GA107M, SM 86, 4 GiB VRAM) now executes
models through a real CUDA-enabled native backend.

- Driver/NVML 620.70.06, CUDA toolkit 13.2.51, host compiler `g++-15`.
- Build: `native/build-cuda`; `libggml-cuda.so.0.25.1` contains SM 86 cubins
  and exports `ggml_backend_cuda_reg`.
- Runtime: `registered_backends()` returns CUDA (1 device) and CPU (1 device);
  CUDA0 reports 3,974,103,040 bytes total VRAM.
- Physical inference succeeded with CPU, hybrid, and full-GPU placement on
  SmolLM2-135M Q4_K_M (30 transformer layers).
- Native readback: 8 requested -> 8 effective (7 transformer + output); 31
  requested -> 31 effective (30 transformer + output). Upstream counts the
  output layer in `n_gpu_layers`.
- Release benchmark (32 generated tokens, same workload): CPU 240.70 tok/s,
  hybrid 289.13, full GPU 484.65. A reverse-order session measured 249.62 /
  309.53 / 488.89 and preserved the ordering.
- VRAM after warmup: about 178 MiB for full offload, 113 MiB for hybrid.
  Unload returns near baseline with about 12.6 MiB retained.
- GPU cancellation, repeated sessions, unload, and VRAM release were exercised.
- GPU load/placement failure is explicit; there is no silent CPU fallback.
- CPU execution remains functional with the CUDA build and with the CPU-only
  build. CPU plans explicitly pin the CPU device and allocate no CUDA buffer.
- Native GPU memory observability exposes per-buffer model/KV/compute totals,
  device total/free/used, sampling timestamps, and explicit unsupported
  counters. Full GPU: CUDA0 model 103,739,904 B, KV 5,898,240 B, compute
  50,921,472 B; hybrid: CUDA0 model 46,928,640 B, KV 1,376,256 B, compute
  50,921,472 B.
- Allocation/deallocation event counters, transfer bytes/duration, peak VRAM,
  process attribution, and synchronization duration remain unsupported.
- Observability sampling costs about 53 µs per snapshot.
- A conservative Qwen2.5-7B Q4_K_M hybrid run (12 layers, 256 context) loaded
  in 2.90 s and generated coherent tokens at 9.30 tok/s, with 2.02 GB of CUDA0
  model buffer, 2.66 GB CPU-mapped model, and 2.30 GB device used.
- Read-only tensor storage introspection identifies every model tensor's
  buffer, device, layer, quantization, file-backed state, and bytes. SmolLM2
  exposes 273 tensors in 2 model buffers; layer 0 is 2,596,608 bytes.
- Controlled staging investigation: measured pinned H2D 6.30 GB/s, pageable
  5.10 GB/s, direct NVMe 1.3–1.8 GB/s. Dense per-token staging is not viable at
  these limits. Graph/scheduler and model-buffer ownership lack per-layer
  allocation/release and boundary hooks. Decision: **READY AFTER SPECIFIC
  NATIVE PRIMITIVES**. See
  [RAMforge/docs/handoff/STAGING_ARCHITECTURE.md](RAMforge/docs/handoff/STAGING_ARCHITECTURE.md).

CPU/GPU greedy text can diverge after the shared prefix due to quantized
accumulation order. Transfer volume, transfer time, GPU utilization, MoE, and
tensor/layer streaming were not instrumented or implemented. Details:
[RAMforge/docs/handoff/GPU_ENVIRONMENT.md](RAMforge/docs/handoff/GPU_ENVIRONMENT.md)
and [RAMforge/docs/handoff/BENCHMARKS.md](RAMforge/docs/handoff/BENCHMARKS.md).

## Current validation entry points

```sh
cd forgeCORE
FORGE_LLAMA_DIR="$PWD/../native" cargo test --workspace --locked

# CUDA build
export FORGE_LLAMA_DIR="$PWD/../native"
export FORGE_LLAMA_BUILD_DIR=build-cuda
export LD_LIBRARY_PATH="$FORGE_LLAMA_DIR/build-cuda/bin"
cargo test --workspace --locked
FORGE_GPTNEOX_MODEL=/path/to/pythia-70m.Q8_0.gguf \
  cargo test -p forge-core --test gptneox_execution --locked -- --ignored --nocapture

cd ../RAMforge
scripts/test.sh --workspace --locked
scripts/smoke-real-models.sh --no-build
RAMFORGE_GPTNEOX_MODEL=/path/to/pythia-70m.Q8_0.gguf \
  scripts/smoke-real-models.sh --no-build --include-gptneox --matrix

# enforced host-memory path (cgroup v2 delegation required)
target/release/ramforge run /path/to/qwen2.5-1.5b.gguf \
  --ram 512M --enforce-ram --ctx 256 --no-repack \
  --prompt "The capital of France is" --max-tokens 1 \
  --temperature 0 --backend forgecore --memory-report --profile
cargo test -p ramforge-runtime --test cgroup_enforcement --locked -- --ignored --nocapture
cargo test -p ramforge-cli --test enforce_ram --locked -- --ignored --nocapture
```

The smoke runner executes the two real models in `models/` sequentially, each
in a separate process, and writes commands, exit status, RSS, GPU state, model
hashes, and output to a temporary evidence directory. It never modifies the
model files. See
[RAMforge/README.md](RAMforge/README.md#real-model-smoke-validation) for the
latest measured matrix.

Current state and limitations are recorded in
[forgeCORE/docs/PROJECT_STATE.md](forgeCORE/docs/PROJECT_STATE.md) and
[RAMforge/docs/handoff/PROJECT_STATE.md](RAMforge/docs/handoff/PROJECT_STATE.md).
