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
development operations.

## Request cost

Use the already resident style-block boundary and borrowed cursor spans.
One allocation-free lexical prefix scan is bounded to 64 KiB; a truncated
or unsafe context conservatively falls back to the original Vue entries.
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

## Validation and remaining evidence

Cover ordinary properties and property-specific values, nested rule/value
contexts, pseudos, at-rules, comments, quoted strings, URLs, `var()` arguments,
custom properties, incomplete edits, CRLF and Unicode. Assert complete useful
Markdown and all original Vue candidate insertion fields/order. Retain exact
raw fixture bytes and the original hover tests in move-only extraction.

Use the existing source Actions and ordinary LSP/performance paths. Compare
repeated completion, resolve and hover against the existing server baseline,
including long comments/strings and a representative large style block.
Record actual latency and available instruction/allocation observations;
The existing Benchmark workflow reuses its identical-profile base/head CLI
builds for repeated real stdio CSS observations. Raw latency samples, response
sizes and exact binary hashes distinguish the previous four-item behavior
from the new product responses. The existing path has no instruction/server
allocation instrumentation; those counters stay unavailable, not inferred.
No speedup, unchanged budget or delivery claim precedes measurement.
Existing full corpus, source checks, instruction ceilings, protected queue
and actual signed delivery remain required. This bounded CSS slice does not
close broad LSP fix history or product readiness issues.

Upstream repositories remain read-only. The retired one-shot SSR trace and
frozen unrelated TypeScript migration are outside this change.
