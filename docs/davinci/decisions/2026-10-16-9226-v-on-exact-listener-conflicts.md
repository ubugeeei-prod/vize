# Preserve actual listener conflicts in use-v-on-exact

Issue: [#9226](https://github.com/ubugeeei-prod/vize/issues/9226).

The Real Project Matrix exposed eight false positives on custom component
listeners: seven `Box` handlers in vue-termui and one `Primitive` handler in
reka-ui. Component events without `.native` do not represent the native listener
overlap checked by this rule. Three lx-music-desktop diagnostics also covered the
handler value instead of only the directive key, producing three false-positive
and three false-negative comparison rows for otherwise matching warnings.

The dedicated fixture retains the complete original SFCs, immutable source URLs,
SHA-256 identities, upstream licenses and all fourteen original Matrix rows.
Forty-two controls cover component/native listeners, namespace boundaries,
dynamic arguments, key alternatives, system subsets and UTF-8 diagnostic ranges.
The independent oracle uses eslint-plugin-vue 10.9.2, ESLint 10.4.1,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0. Complete provider
packets are retained before classification; only their ephemeral file path is
replaced in the committed receipt. Provider entry points, manifests and the rule
implementation have recorded hashes.

Two reference omissions are explicitly retained as correct Vize findings:

- `@keydown.alt.shift` also accepts the Ctrl+Alt+Shift event accepted by
  `@keydown.alt.ctrl.shift`. Sorted string containment incorrectly misses this
  system-key subset.
- `@keydown.up.down` and `@keydown.ctrl.down` both accept Ctrl+ArrowDown. Keyboard
  alternatives overlap even when their complete key lists differ.

Chromium 152.0.7977.65 delivered trusted `KeyboardEvent` inputs to actual Vue
3.6.0-beta.10 compiled templates. In both cases the original event invoked both
handlers; adding `.exact` to the broader handler invoked only the specialized
handler. The four complete browser observations are retained in
`runtime-controls.json`. The Node test also replays those observed event fields
through the official Vue compiler's generated callbacks. These two findings
remain visible as explicit correctness exceptions to the reference packets;
neither the shared comparator nor its baseline is changed.

The Rust integration test checks all 45 complete sources using the public SFC
linter and verifies diagnostic counts, severity, rule identity, absence of fixes,
and exact UTF-8 key spans. Run its source build in Actions alongside the normal
affected-crate checks. The independent packet and runtime replay can be checked
without a native build:

```sh
node --test tests/tooling/patina-use-v-on-exact-oracle.test.mjs
```

The oracle's `--record` mode preserves raw receipts in a temporary directory and
refreshes reference packets only after running the pinned providers. It preserves
the two explicit Vize expectations rather than deriving them from an upstream
blind spot.
