# Current ref reads in immediate computed maps

This decision addresses the third-party [#8756 report](https://github.com/ubugeeei-prod/vize/issues/8756)
and retains the [#7244 computed-spread history](https://github.com/ubugeeei-prod/vize/issues/7244).
The report's entire `ComputedLocal.vue`, configuration, tsconfig and issue body
live in `tests/_fixtures/differential/lint-regressions/computed-local-reactivity-8756/`.
The `.vue.txt` carrier has a real Croquis/parser and source-CLI regression consumer;
it does not change the independently owned physical `.vue` acquisition census.

## Cause and bounded correction

The existing ref extraction and spread checks already exempt their original
single-function execution scope. They reject a second function because it may
run after its surrounding callback returns. The report's inline `map` callback
adds exactly this second function even though the builtin array operation calls
it immediately while the computed getter is executing.

The new exception admits only a fresh array literal containing primitive literals
with no holes, a direct nonoptional `.map` call with one inline nonasync arrow argument,
no generic call type arguments, and an active chain of admitted map arrows
reaching an authenticated Vue
`computed` or `watchEffect` arrow. The actual value-import module, exported name,
local binding, declaration and source spans establish Vue authority. A complete
binding census refuses lexical and later hoisted shadows; imported aliases retain
their actual exported identity. Const aliases, namespace APIs, computed options,
regular functions, arbitrary/custom `.map` receivers and effectful array elements
remain outside this added exception. Their historical behavior is retained.

Private optional span frames are pushed and popped around the existing argument
walk. They neither change public scope kinds nor enter Croquis/debug payloads.
The existing whole-program top-level-await visitor also gathers callback facts
when diagnostics, literal `.map` spelling and Vue-source candidates demand them.
Compile demand and map-free inputs retain the original await-only traversal;
there is no additional parser or pipeline stage. The original visitor extraction
is kept in a move-only commit before this behavioral addition.

For the added map exception, a spread of an existing snapshot is accepted only
when its ref/property origin was declared inside that reactive owner's scope.
Captured setup snapshots and plain aliases remain warnings. The existing
single-function exemption, provenance recording, mutation checks and diagnostic
translation retain their original behavior. A standalone top-level map arrow
has only one function scope, so that original broad exemption remains intact:
its control is classified explicitly as preserved historical behavior with an
expected zero count from the existing source predicate. It supplies no new
reactive-callback authority or runtime qualification. Auditing that existing
single-function policy beyond this issue remains separate unfinished work.

## Intrinsic and asynchronous boundary

The exception assumes the standard unmodified ECMAScript array intrinsics and
indexed prototypes in the executing environment. It does not prove arbitrary imported code or external
global monkeypatches absent. The whole current script refuses this added
exception if it exposes intrinsic replacement paths: `Array`, `Object`,
`Reflect`, global objects, dynamic code, prototype/constructor accesses,
computed property access or map-member assignment. This conservative scan
includes nested bodies, escaped identifiers, late statements and deferred
prototype changes, so source-visible assignment, accessor and deferred override
counterexamples remain warning-positive. Unrelated code can conservatively
prevent this new exception; it never creates additional exemptions.

Async owners/arrows and unknown/deferred nested functions do not carry the new
authority. The standard [array map algorithm](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.map)
calls each callback within the operation. Vue's [watchEffect documentation](https://vuejs.org/guide/essentials/watchers.html#watcheffect)
limits dependency tracking to synchronous execution. These facts justify the
finite admitted execution chain, without claiming async continuation tracking.

## Regression and delivery obligations

The report supplies historical v0.443.0 warning evidence for the two original
findings; no locally rebuilt baseline is claimed. The new Croquis tests consume
the whole reported carrier and all 35 whole authored controls. Positive controls
cover computed, watchEffect, value-import aliases, nested literal maps and a
fresh owner-local read. Negative controls cover ordinary callbacks/functions,
foreign and shadowed computed calls, async/await, regular/deferred/Promise
callbacks, custom/aliased/spread/effectful/optional/hole receivers, source-visible
intrinsic overrides, captured outside snapshots/aliases and later setup reads.

The existing tooling campaign consumes the exact same 36 SFC inputs through its
authenticated source-built CLI and original opinionated/type-aware config.
The existing installed native TypeScript 7.0.2 provider is supplied explicitly
through `CORSA_BIN`; whole package identity, binary hash and raw version probe
are retained. The temporary project links the existing test workspace's Vue
modules, recording its package body and declaration hash; there is no install.
Complete source and raw stdout/stderr/status/signal/error packets are retained
before assertions. No unrelated binary fallback, extra builder or manual
qualification stage is introduced. The original #7244 direct/deferred-spread
test, all rule/preset/budget/default vectors, protected instruction ceilings and
full qualification remain unchanged.

Source review, genuine Actions runtime regression, all required protected
checks, actual merge and released installed acceptance remain pending at this
authored-source stage. An open PR or a merged source fix alone does not complete
the external issue. Reporter credit uses the verified public signed
[5e929628 commit](https://github.com/naitokosuke/chirashitabi/commit/5e92962832595edf50c48ca7f24a8c7fed1f4e86):
account 102337893, author ナイトウコウスケ,
`102337893+naitokosuke@users.noreply.github.com`. It is not inferred from the login.

The first real source-CLI campaign at `37080859` reached the authored async
owner and correctly returned exit 1 with the existing
`script/no-async-in-computed` error plus both snapshot/spread warnings.
The consumer had incorrectly required exit 0 for every input. Its finite
correction records that complete error packet for the two direct async owners,
checks their exact error diagnostics/count and exit 1, and retains exit 0 with
no errors for every other input. All 36 source bodies, the original opinionated
config, the full snapshot/spread counts and every rule remain unchanged.
The original failure is retained; fresh source and protected runs are required.
