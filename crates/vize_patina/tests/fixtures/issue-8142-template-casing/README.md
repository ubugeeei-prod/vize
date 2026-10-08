# Issue #8142 template casing corpus

These 50 SFC inputs were authored independently for the registered-component
default correction. They contain no copied n8n branch source. Upstream remained
read-only.

The complete independent oracle packets are preserved byte-for-byte:

| Packet | Cases | SHA-256 |
| --- | ---: | --- |
| `oracle-paired.json` | 4 | `6193520dab51cda3fd75937f3a50b427ba6140903cfc4dea519c3dc43d47c4d8` |
| `oracle-registration.json` | 17 | `64250dc3b5c7257217b52478e6318a5f29832a35a0dbf1d841eb72518a444b42` |
| `oracle-registration-additional.json` | 11 | `6f34cc850b933c3304a22826fa18a15475531c48fc4bdf3609777bbd43ef47fa` |
| `oracle-options.json` | 12 | `f176fd1d97d16033a259ce73b7c820195d454808f41f844bad5152f8565f71f4` |
| `oracle-boundaries.json` | 6 | `cb49abe4278051da1a1aa2e91e0895981d574a04e13984e8fd2308b4076a9edf` |

ESLint 10.4.1, eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and
@typescript-eslint/parser 8.65.0 ran on Node 24.14.0. Packets retain exact
configuration, original sources, complete diagnostic/fix objects and provider
entry hashes. Registration-only and option packets contain two matching actual
observations. Absolute provider paths identify the original local capture;
the regression reads only the committed JSON and never those paths.

`source-before.json` is the complete observed Vize public `Linter::lint_sfc`
result at `b8ea16711117c502b14724e1eb1e4a7022a7f1d7`. A temporary integration
observer used the incremental preset, exactly the casing rule (plus
`vue/require-component-registration` for the paired four), default severity and
full help. It ran with Rust 1.99.0 through:

```sh
cargo test -p vize_patina --profile ci --test template_component_registration capture_original_source_behavior -- --nocapture
```

The observer serialized every diagnostic field, including labels and fixes,
plus filename and both counts. The original temporary paths are retained as
capture provenance; this receipt establishes source API behavior, not an
authenticated installed package or shipping CLI.

`source-after.json` freezes the intended correction: remove casing findings only
where the independent default oracle reports none, preserving every other source
field and registered-positive fix byte. `options-after.json` freezes complete
source expectations for the separate explicit false/globals and authored-definition
boundary controls. The test
compares the complete actual source envelope, then compares registration
membership and final bytes with every independent case. No rule-status filter,
fallback preset or native completion credit is used.

The original #7905 authored23 standalone edit cases retain their old behavior by
explicitly selecting the all-tag policy; their existing fixture bytes and oracle
remain unchanged.
