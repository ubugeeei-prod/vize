These TypeScript declaration fragments are included verbatim in the checker's static helper streams. The `.d.ts.txt` suffix distinguishes emission inputs from standalone declaration files. Whitespace, indentation and terminal newlines are intentional because they contribute to generated mapping positions.

`core-types` and `prop-keys` are shared by Vue 2 and Vue 3. The remaining fragments retain the dialect or emission scope that owns them. Rust composes the streams with `include_str!` and `concat!`; emission performs no source-file reads, parsing or template replacement at runtime.

Declaration changes must qualify the existing generator, mapping, native checker and Content Mapper tests. Update current projections when behavior changes; retain historical evidence and account for any offset changes explicitly.
