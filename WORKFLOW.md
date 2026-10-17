# RAMforge + forgeCORE shared Arena workflow

This directory is the shared development workspace for RAMforge and forgeCORE. Both projects are developed together in the same Arena chat.

## Source and Git boundary

- Arena edits source files directly in this Git-free workspace:
  - `RAMforge/`
  - `forgeCORE/`
- Git operations are user-controlled and are performed manually and infrequently in separate external clones.
- Arena must never perform commits, pushes, pulls, branch changes, resets, rebases, merges, checkouts, fetches, or any other Git history, index, branch, or worktree operation.
- Arena must never claim that work has been committed, pushed, or synchronized with a remote.
- Before each implementation task, inspect the current state of both projects, review recent changes, identify regressions and cross-project compatibility issues, and do not rely on stale assumptions.
- After each change, report exactly which files changed and why, the checks that were run and their real outcomes, unresolved risks, and commands the user may choose to run manually.
- Do not change dependency pins or native-library revisions without explicit technical justification and user approval.
- Do not perform broad formatting rewrites, large native rebuilds, or unrelated refactors as incidental cleanup.

## Ownership and dependency boundary

- RAMforge depends on forgeCORE through forgeCORE's public safe Rust API.
- RAMforge must not directly call llama.cpp/ggml FFI or depend directly on `forge-sys`.
- RAMforge owns orchestration, budgets, accounting, residency, cache and eviction, scheduling, prefetch, placement, and diagnostics.
- forgeCORE owns safe Rust abstractions over native FFI, native handles, ABI/layout, model/context/batch/graph interfaces, and backend/device/buffer primitives.
- llama.cpp/ggml owns GGUF/native model loading, architecture implementations, quantization, kernels, and graph execution.
- Keep Legacy RAMforge execution available until forgeCORE parity and replacement requirements have been demonstrated. Never silently fall back between backends.

## Change and validation rules

- Preserve manifests, lockfiles, tests, documentation, fixtures, configuration, scripts, and relevant source assets unless a task explicitly authorizes a change.
- Do not silently overwrite, discard, normalize away, or delete uncommitted, untracked, ignored, local-configuration, fixture, or build-related information.
- Do not copy Git metadata into this workspace and do not modify the external clones or original source directories.
- Do not alter dependency pins, lockfiles, native versions, model fixtures, or project behavior during a workspace-only transition.
- Lightweight inventory, checksum, source, and formatting checks are preferred before expensive work. Do not claim compilation, tests, Clippy, native integration, commit, or push success unless actually performed.
- Native dependencies and build outputs remain external to this workspace unless a task explicitly establishes otherwise; do not delete or rebuild them incidentally.
