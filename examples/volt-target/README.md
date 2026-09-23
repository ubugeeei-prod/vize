# Volt output-target exercise

A guest for the P6-1c `output-target` world. It is not Volt. Given the
committed S2 page `Hello` and S3 page `render`, it emits one HEEx module
whose `Hello` link points back at the S2 text.

Recorded command, from the repository root, on the workspace toolchain
(`rust-toolchain.toml`, which provides `wasm32-wasip2`):

```bash
cargo test -p vize_extension_host --offline --features extension-host --test volt_target
```

The host runs that guest out of process and in process and accepts the
document through `OutputSession`. The contract-change list is
`davinci-road/plan/phase-6-records/p6-6.md`. The Volt maintainer has not
signed off; the phase index stays open.
