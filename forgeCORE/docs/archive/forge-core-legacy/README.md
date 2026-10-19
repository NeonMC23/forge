# Archived forge-core legacy implementations

This directory preserves historical, inactive `forge-core` implementations
from before the ggml pivot. They are source material only and are not
supported production APIs.

- `attention.rs` — legacy scalar single-token attention implementation.
- `kv.rs` — legacy pure-Rust per-layer KV-cache implementation.

Both files are outside the active Rust module graph. Their embedded
`#[cfg(test)]` tests are preserved as historical source, but are not part of
the current Cargo test graph and are not compiled or tested by Cargo.

Do not restore, port, or adapt these implementations without a separately
approved task.
