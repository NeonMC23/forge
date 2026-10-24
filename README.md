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

## Current validation entry points

```sh
cd forgeCORE
FORGE_LLAMA_DIR="$PWD/../native" cargo test --workspace --locked

cd ../RAMforge
scripts/test.sh --workspace --locked
```

Current state and limitations are recorded in
[forgeCORE/docs/PROJECT_STATE.md](forgeCORE/docs/PROJECT_STATE.md) and
[RAMforge/docs/handoff/PROJECT_STATE.md](RAMforge/docs/handoff/PROJECT_STATE.md).
