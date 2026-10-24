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
| `gptneox` / `gpt_neox` | not-yet (explicit, actionable rejection) | yes | Pythia-70M-deduped Q8_0; native and independent NumPy parity |
| `qwen3`, `qwen35`, `gemma`, `phi`, literal `mistral` | not-yet | not-yet | inspect/plan only |

`gptneox` execution must be requested explicitly with `--backend forgecore`; the
legacy path never falls back or approximates it. The real 20B GPT-NeoX GGUF is
inspect/plan-only and was not run: forgeCORE loads weights eagerly, does not
enforce RAM budgets, and the host has 15.39 GiB RAM. See
[RAMforge/README.md](RAMforge/README.md#architecture-and-quantization-matrix)
for the full architecture/quantization matrices and the measured budget table.

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
