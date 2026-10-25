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
target mapping (`A 8192->0, B 8192->8192`). The 4G/1G/100M 20B cases were not
run. See
[RAMforge/README.md](RAMforge/README.md#enforced-host-memory-budgets-cgroup-v2)
for the enforced matrix, performance, and limitations.

## Current validation entry points

```sh
cd forgeCORE
FORGE_LLAMA_DIR="$PWD/../native" cargo test --workspace --locked
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
