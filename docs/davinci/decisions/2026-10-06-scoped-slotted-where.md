# Scoped functional and slot selectors

Issue: [#7942](https://github.com/ubugeeei-prod/vize/issues/7942).
Reporter: ubugeeei (GitHub ID 71201308).

A flat style block containing `:slotted()` selects the existing Vite selector
writer. That writer inserted scope attributes before a bare `:where()` and
scoped ordinary compounds preceding slotted content. This changed specificity
and broke the original two-rule component.

Keep this selection and correct the writer. A leading `:where()` or `:is()`
recursively scopes its branches while it remains the scope anchor; a later
ordinary class, type or attribute owns the anchor instead. Slotted targets use
the `-s` identity inside that same functional anchor. Ordinary prefixes before
`:slotted()` remain unscoped. Existing deep/global handling stays on its current
route. This uses the current selector helpers and emits in the current pass.

The primary reference is [Vue 3.5.38 pluginScoped.ts](https://github.com/vuejs/core/blob/v3.5.38/packages/compiler-sfc/src/style/pluginScoped.ts).
Its `rewriteSelector` clears outer injection at `:slotted()` and recursively
rewrites the selected `:where()`/`:is()` node and the slotted argument. A class
before the functional pseudo or after it prevents that pseudo from owning scope.

The original complete issue body and SFC are retained in
`tests/_fixtures/differential/compiler/scoped-slotted-where-7942`. Four original
CSS table inputs and four authored anchor/list/escape controls have complete
selector and declaration references. The old two focused test files are
archived byte-exactly. Their original inputs remain unchanged; only the
incorrect ordinary-prefix attributes in the expected slotted CSS are removed.
This correction follows the primary selector contract, without recording Vize
output as an expectation.

The existing attribute writer was extracted in a separate move-only commit.
The parent oversized source shrinks; the new module and tests stay below 350
lines. The genuine per-crate consumer inventory is regenerated over the full
SFC crate with other shards untouched.

Prepared checks cover the whole pipeline CSS, both minified/non-minified
complete Rust and public NAPI CSS results against authored scoped references,
the original whole SFC through the public Rust and NAPI APIs, complete repeat
results and additive original-source maps, and the real Vite CSS wrapper.
These public native-addon checks exercise its existing SFC compiler route;
they grant no level-native acceptance beyond the actual admitted native scope.

Hosted exact-head source checks, the full existing compiler corpus, protected
instruction gates, actual signed merge and an installed release replay remain
pending. No benchmark/ranking or performance result is claimed. Root owns
publication and the public replay handoff.

The first exact-head source build at `10b9afef` failed before tests: Rust 1.98
Clippy rejects six unchecked string slices in the new functional helper. The
correction uses the existing pseudo-function parts and checked UTF-8 access.
All original sources and complete expected CSS/API vectors remain unchanged;
the failed build is retained and grants no execution acceptance. The separate
inherited dependency audit failure remains with the security lane. Fresh
exact-head source and protected checks are still required.

The same first tooling attempt failed before the new assertions because the
tests package does not declare `@vizejs/native`. Its direct import now uses the
existing checkout-native entry, matching other native tooling tests. The real
Vite wrapper still resolves its declared native dependency. The unchanged
registered task builds that checkout addon first. The complete 872,085-byte
failed tooling log is retained; this repairs test provisioning without changing
any compile option, assertion, input or expectation.
