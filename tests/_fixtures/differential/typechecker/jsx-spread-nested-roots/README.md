# Nested JSX in a spread child: original Element Plus consumer

`tab-nav.tsx.txt` preserves the complete authored Element Plus file at commit
`20298a21ae54a5417955373b80130ae7c548a6bb`. `tab-nav.original-h.ts.txt`
preserves its complete original Vize plain-TypeScript projection at
`605429c660b328bb03bec5bbfef15961e59fcaf0`, captured during actual native
execution in Actions run 38115632497. `original-custody.json` binds all bytes,
producer, upstream pin and provider archive; `LICENSE` retains the MIT license.

The original spread child contains an array and conditional with a nested
`<TabBar/>`. The authored TSX parses, but the original `.tsx.ts` projection
retains that JSX and fails plain-TypeScript parsing. The regression regenerates
the whole authored module, requires complete plain-TypeScript parse success,
and retains the semantic component call and original argument source maps.
Another control retains callback scope and a genuinely incompatible nested
component prop expression for native type checking.

Only analysis lowering retains the missing nested root. Existing compilation
and unsupported runtime-spread verdicts remain authoritative. Run
`cargo test --locked -p vize_canon --lib jsx_codegen::nested_tests::`.
The original full consumer suite and native completion qualifier are separate
required evidence.
