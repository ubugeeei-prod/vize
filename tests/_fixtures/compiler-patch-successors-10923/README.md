# Complete compiler patch-oracle successors

The original expected compiler hashes remain in the patch oracles. These files
retain complete previous and current packets for two independent compiler changes:

- Vue Router's clean imported callback uses `_unref(navigate)` in both the guard
  and call. Every other code byte and packet field remains equal.
- Failed-template script setup calls the default `__expose()` introduced by
  #10923. Every diagnostic, location, warning, filename, language, CSS and macro
  artifact remains equal. The clean create-vue output is completely unchanged.

`capture.json` records the exact upstream revisions, complete input hashes,
public npm package versions, entrypoint and loaded native-image hashes, and raw
packet hashes. Previous `vize` and `@vizejs/native` are 0.443.0; current packages
are 0.600.0. The clean packet files are verbatim output files from the published
CLI. Previous Vue Router clean and broken packets reproduce the existing oracle
hashes `84e9feaa...` and `0914134e...` exactly.

The CLI correctly refuses to emit a file for broken input. Broken captures retain
the complete published `compileSfc` result separately and project its actual
code/CSS/errors/warnings/macro artifacts into the unchanged SFC oracle DTO, with
the filename and TypeScript language supplied by the byte-pinned input. This
projection is stated explicitly; it is not a claim that a failed CLI emitted a
module or that a published package ran the internal Rust oracle executable.

The helper verifies full packet equality with only the specified code edit,
then compares every byte of the actual current oracle output with the complete
successor. Mutation tests reject unrelated code, diagnostics, metadata, hashes
and raw serialization changes. This admission grants no additional runtime,
whole-product native parity or performance credit.
