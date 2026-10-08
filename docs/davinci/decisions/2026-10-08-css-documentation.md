# Static CSS completion and hover documentation

Refs [#3953](https://github.com/ubugeeei-prod/vize/issues/3953),
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Product behavior

The previous style provider offered only four Vue CSS features: `v-bind`,
`:deep`, `:slotted` and `:global`. Its module documentation claimed CSS
properties, but ordinary property, value, pseudo and at-rule suggestions did
not exist. The user explicitly requests those ordinary CSS surfaces, with
useful descriptions and careful performance.

Add a compile-time static standard CSS catalog. Property completion selects
matching property names; value completion uses the owning declaration's
catalog values. Available pseudo classes, pseudo elements and at-rules use
that same static authority. Complete Markdown explains the selected entry,
shows syntax and a relevant example, and retains its authoritative MDN/spec
references. Unknown custom properties keep their existing behavior and never
borrow another property's value documentation.

Keep the four original Vue candidates in their original order with the same
labels, kind, detail, insert text, snippets, ranges and edits. Enrich their
selected documentation and correct two prose errors: the left anchor of a
`:deep()` selector receives the scope attribute, and CSS `v-bind()` also works
in an unscoped style block. The explicit Vue documentation section links are
the authority; this is presentation work, not compiler migration evidence.

## Static source

Pin Microsoft's `vscode-css-languageservice` commit
`2a253759802c9aba0d71b30e369fb65f1f062b5c`, file
`src/data/webCustomData.ts`, Git blob
`6363c511b09a58747a5f31829d74dd772f840f0d`.
The complete 1,035,084-byte source has SHA-256
`b1f6d6250bfd86d640abda79046fedcb94123f127d47398b6ad5ca8b68c4464b`.
Preserve the whole offline data snapshot, license and attribution; generated
Rust lookup tables follow the source provider's first-name-wins policy.
The source contains 888 properties, 2,148 property-owned values, 25 at-rules,
111 pseudo-class rows and 90 pseudo-element rows. Duplicate names remain in
the frozen input and produce 110/88 effective pseudo entries respectively.
The separate pinned named-color source retains 148 named hex colors plus
`currentColor` and `transparent`. Five CSS-wide keywords and `var()`/`calc()`
use the linked W3C specifications. Generation and verification are offline
development operations. The immutable `web-custom-data.json.txt` fixture keeps
the original source JSON bytes; normal repository formatting applies to
handwritten generators and manifests, with no new formatting exclusion.

## Request cost

Use the already resident style-block boundary and borrowed cursor spans.
One allocation-free lexical prefix scan is bounded to 64 KiB; a truncated
or unsafe context conservatively falls back to the original Vue entries.
An intentionally complete declaration-head slice ends at the cursor, so
adjacent whitespace still permits an empty property prefix. The separate
right-hand lookahead retains its conservative 1 KiB truncation guard.
Do not add a full stylesheet parser, document analysis stage, request-time
JSON parsing, network requests or a native type-checker query.

Prefix-filter static catalog entries before creating LSP items. Completion
resolve creates complete documentation only for the requested immutable
catalog identity; hover builds only the selected entry's Markdown. Keep
insertion and replacement fields intact during resolve. Lazy documentation
also applies to the original Vue entries wherever resolve is supported;
unsupported resolve configurations retain useful direct documentation.
Initialize records the client's exact `resolveSupport.documentation` support
in one conservative per-server flag, independently of typecheck configuration.
Style-only hover/completion bypass Corsa before initialization; selected CSS
resolve runs before all native typecheck routing. Property replacement ranges
are computed once per request and shared by the prefix-filtered items.
Completion consumes the same owned text snapshot used to locate its cursor,
removing a second whole-SFC copy while retaining completion insertion boundaries.
The original Vue suggestions and hover remain available in other style dialects;
only the new standard catalog requires CSS, SCSS or Less syntax.

## Validation and remaining evidence

Cover ordinary properties and property-specific values, nested rule/value
contexts, pseudos, at-rules, comments, quoted strings, URLs, `var()` arguments,
custom properties, incomplete edits, CRLF and Unicode. Assert complete useful
Markdown and all original Vue candidate insertion fields/order. Retain exact
raw fixture bytes and the original hover tests in move-only extraction.

Use the existing source Actions and ordinary LSP/performance paths. Compare
repeated completion, resolve and hover against the existing server baseline,
including long comments/strings and a representative large style block.
Record actual latency and available instruction/allocation observations.
The existing Benchmark workflow reuses its identical-profile base/head CLI
builds for repeated real stdio CSS observations with lazy and eager clients.
Eager clients request completion and hover; only clients advertising resolve
support request and measure completion resolve.
Keep the original eight scenarios and also measure an empty prefix with all
candidates and 48 KiB unterminated comments/strings with bounded fallback.
Malformed observation cursors remain inside the style body; the original
strict style-end boundary is preserved.
Raw latency samples, response sizes and exact binary hashes distinguish the previous four-item behavior
from the new product responses. The existing path has no instruction/server
allocation instrumentation; those counters stay unavailable, not inferred.
No speedup, unchanged budget or delivery claim precedes measurement.
The [initial ordinary Benchmark run](https://github.com/ubugeeei-prod/vize/actions/runs/37778966280)
passed for base `2f9fe178ab7ce2d66a17c9e7fe84570cc86cfbb5` and
head `e774474f5c46c2f02b15c4c77045f3d2f679b217`, using five warmups and
80 samples per operation. The head's completion p95 was 0.0943 ms for the
48 KiB closed comment, 0.0932 ms for the closed string and 0.0869 ms for
700 rules. Corresponding response sizes were 5,256/5,346/5,316 bytes,
versus the baseline's 1,302-byte four-feature response. Ordinary CSS hover
was absent in that baseline, so these are observed product costs, not an
equal-feature speedup. This earlier source's full Check failed on test lint
and stale move inventories; it is not delivery qualification. Fresh full
source checks and the expanded eager/empty/malformed observations remain
required for the successor.
Existing full corpus, source checks, instruction ceilings, protected queue
and actual signed delivery remain required. This bounded CSS slice does not
close broad LSP fix history or product readiness issues.

Upstream repositories remain read-only. The retired one-shot SSR trace and
frozen unrelated TypeScript migration are outside this change.

## Stack delivery

The CSS child follows HTML documentation PR [#8316](https://github.com/ubugeeei-prod/vize/pull/8316),
rebased onto its actual `e21cf419fab928c8538e0b5f2ac1acd3d9e06e2c` source
after its earlier queue conflict was removed. That genuine parent descends
from actual signed main `81aa0c449cdeb2daf2959d3c9a272ff880bf4765`.
The earlier `85f0240debf14ea810782045840a3f26a84ec59d` CSS source is retained
with its real test-macro, empty-prefix and malformed-observation failures;
it supplies no qualification credit to this refresh.
The next `d2ae8fbd3f5455915a345a39c51da3fa3b72c799` source's CSS
stdio and Benchmark observers exposed an assertion mismatch in the new
malformed-context scenarios: the selected `v-bind` completion's documented
signature is `v-bind()`. Assert that complete signature rather than changing
the preserved Vue Markdown. Retain all four fallback items, exact resolve
packet preservation, useful reference documentation and null malformed hover.
Fresh qualification remains required after this observer-only correction.
A subsequent source review found that the new CSS completion fast path
preceded the existing JSX opt-in guard. The resident SFC parser can classify
a literal React `<style>` element as a style block. Restore the unchanged
`.jsx`/`.tsx` guard before CSS routing, matching hover precedence, and assert
real completion/hover remain absent with JSX typechecking disabled even when
the resident context is a style block. Ordinary Vue CSS still bypasses Corsa.
The actual native Maestro regression exits 101 before the precedence move
and passes afterward. The unchanged missing-Corsa CSS handler control and
completion snapshot/every-cursor law also pass on the corrected real crate.
These local regression observations do not replace fresh hosted source and
performance qualification.
Root owns native Stack registration and
protected admission. After a parent prefix actually merges, rebase and retarget
any remaining child onto fresh main, then rerun its source Actions. Earlier
parent or CSS measurements do not qualify a new combined source.
