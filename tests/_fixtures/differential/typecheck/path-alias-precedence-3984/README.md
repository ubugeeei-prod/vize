# Authored path alias precedence

This is a bounded regression corpus for [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).
It is authored input, with independent TypeScript 6.0.3 and native Corsa 7.0.2
oracles; it is not attributed to an external project or reporter.

`App.tsx.txt` and the same script in `App.vue.txt` require a string from `@x`.
The exact and wildcard targets deliberately export different primitive types.
The complete expected diagnostic packets are fixed in the corpus driver;
they are never generated from the changed Vize producer.

The eight cases cover:

- Exact key priority over `@x*`, including the inverse diagnostic and repair.
- Exact key priority over the equal-scored `@*` route.
- A missing first target followed by a valid target of the same exact key.
- A selected exact key with all targets missing, preserving TS2307.
- A selected longer wildcard with all targets missing, preserving TS2307.
- Unaffected exact-only clean and broken controls, retaining whole CLI reports.

Both source CLIs consume identical files. The public `check-server` executes
the actual native project and returns the full diagnostic packet and virtual
TypeScript; the opt-in TSX CLI reports the complete project result. The same
TSX project also runs through stock TypeScript and the actual native compiler.
The driver retains every command, input hash, full reply, stdout, stderr,
process status, executable hash and source SHA in the Actions artifact.

The baseline manifest preserves the observed six wrong results from public
v0.439.0. Correct expectations come from the authored type contract and both
original engines. A local source/native pass is not a public release pass.

The existing paired benchmark separately checks unchanged diagnostics and
programs on its fixed 500-SFC workloads and nine alternating base/head pairs.
No 10x claim is made here.

Equal-prefix wildcard ties retain incoming alias order. Full authored JSON
declaration order after config flattening, wildcard targets without `*`, and
broader project/build/watch/emit requirements remain separate TODOs.
