# Options API computed regression fixtures

Tracked in [#6879](https://github.com/ubugeeei-prod/vize/issues/6879) and
[#6921](https://github.com/ubugeeei-prod/vize/pull/6921). Preserve the original
writable-computed fix and its repeated/exported mixin precedence tests.

Extract computed writability and default-export classification into child
modules in a move-only commit. The parent returns below 350 lines; the moved
bodies preserve behavior. Keep the already-fixed recursion-path removal rather
than bypassing the unresolved review thread.

The private parse-sharing candidate derives default-export spans, writable
computed names and unresolved-extends status from one existing OXC script parse
and the same parsed Options object. Binding emission borrows those owned facts;
it adds no pipeline stage or serialization. Options facts are collected only
under the existing Options API template-binding gate, and setup-only rewrite
classification remains disabled. Existing rewrite guards, merge precedence,
repeated mixins, recursion-path removal and authored emission/mapping bodies are
preserved. The runtime-props helper's existing separate parse is outside this
small change. Joint-result tests cover export spans, repeated/exported mixin
precedence, unresolved extends, independent rewrite/binding gates and export
whitespace/comment controls that keep computed analysis unconditional. Source
compilation and exact diagnostic execution remain pending existing PR Actions.
The owner's later `5e0048c0` computed/prop-collision fix is preserved in a
separate private compatibility commit: its resolution body and regression test
remain byte-exact, with only the duplicate parse entry replaced by shared inputs.

Preserve the actual later owner `13a694c1` ancestry, wrapped `props`/`mixins`
arrays, authored/camelCase prop spellings and statically absent setter handling.
Move its complete wrapped-options test body unchanged into an ordinary child
module so the parent stays below 350 lines. Setter classification looks through
OXC's erased parenthesis/TypeScript wrappers and distinguishes the builtin
`undefined` from direct or block-hoisted module bindings using the already-parsed AST. Unrelated local
function bindings do not shadow module-level descriptors; authored references
and calls otherwise retain the existing checker-dependent behavior.

[Vue's setter dispatch](https://github.com/vuejs/core/blob/v3.5.30/packages/runtime-core/src/componentOptions.ts#L639)
requires a callable value. Local Vue 3.6.0-beta.10 and TypeScript 6.0.3 execution
confirmed that wrapped `null`/builtin `undefined` warn and never invoke a setter,
while a callable module binding named `undefined` is invoked. A separate strict
TypeScript module accepted that binding without diagnostics. These observations
establish the intended semantics, not candidate Rust execution.

The second registered corpus, `options-api-computed-setters`, retains the
owner's wrapped-options input and adds wrapped absent/callable setters, an
unrelated local binding and direct/block-hoisted callable module shadowing. Its source-built Rust
virtual-TS witness compares complete declaration vectors including mutability.
Fresh Actions must execute these oracles. No CLI diagnostic text/position is
guessed, and this corpus adds zero native comparisons or L2/L3 sweep baselines.

The authored `tests/fixtures/typechecker/options-api-writable-computed` case
registers the existing SFC, compiler options, provenance and complete diagnostic
reference through the original Rust CLI integration test. A writable setter
produces no assignment diagnostic; the getter-only assignment retains its
severity, source position, TS2588 code and full message. The oracle compares
the entire diagnostic sequence and exit status, with no assertion-lint exemption.
Resolve Vue from the existing root/tests/playground/examples/nuxt package paths
and link its `@vue` namespace. `VIZE_TEST_REQUIRE_TSGO=1` fails on missing Vue;
the explicit T0 engine opt-out remains deferred. Original source CI finished the
CLI test in 0.00s with its root-only lookup, so it provides no runtime credit.

This explicitly registered typechecker fixture does not enter automatic
`tests/_fixtures` L2/L3 sweeps. Their baselines are not changed without execution.
The future shared typechecker differential executor remains tracked by #6852
and #6879; native comparisons are zero. No native migration or fix-history
closure is claimed by this legacy regression.

Local preparation verifies formatting, preserved moved bodies and fixture
membership/hash metadata. The complete exact diagnostic oracle must be checked
by the existing source-built CLI/TSGO Actions gate after publication. Review
threads and latest-head checks remain required before auto-merge; no bot review
is dismissed or bypassed.

The two registered fixture directories join the existing formatter-sensitive
fixture policy: fresh source Actions reported seven formatting-only failures,
and formatting would change authored source positions and golden diagnostic
bytes. Lint, assertion and source-length policies remain fully enforced.

Three new lexical setter controls deliberately shadow builtin `undefined`;
HoistedSetter also executes an `if (true)` nested var declaration. Precise
next-line lint annotations explain those control inputs without changing any
root lint rule. Their source digests are refreshed; all complete binding vectors
and the author’s WrappedOptions/WritableComputed inputs remain unchanged.
