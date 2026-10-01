# One markup state machine behind the published tokenizer

Issue: [#6835](https://github.com/ubugeeei-prod/vize/issues/6835).

The ordinary-build `Tokenizer<C: Callbacks>` now owns one
`Lexer<Component, CompatSink<C>>` or `Lexer<Document, CompatSink<C>>`. Its six
published methods retain their signatures. Armature and SFC duplicate-attribute
consumers continue through their existing parsers and callback owners. This
change retires the duplicate lexer implementation, not those product parsers.

`CompatSink` is available by default. It preserves the first scalar delivered
by legacy entity callbacks, all 21 diagnostic mappings, quote vocabulary and
live callback-controlled verbatim polling. Native sinks still receive the
complete decoded value with its authored span. The published
`try_decode_entity` helper delegates to the same complete-value decoder and
returns the preserved first scalar and consumed byte count. No level gains a
normal or build dependency on a legacy product.

## State and configuration

`set_tolerate_declarations` safely moves the owned lexer between Component and
Document specializations. Exhaustive destructuring transfers every cursor,
state, sequence, delimiter, entity, raw-interpolation and callback field; adding
a field requires an explicit transfer. An internal `Option` permits the owned
move without cloning the callback or using unsafe code. Repeated `tokenize`
calls retain the lexer state, including the original repeated EOF cleanup.

The raw-interpolation switch checks the default **opening** delimiter. A custom
closing delimiter retains its existing behavior. Empty and arbitrary byte
delimiters, repeated setters, live mode polling and an interrupted callback
followed by a non-EOF profile change remain covered. Document selects declaration
tolerance in the lexer; its complete host tree-construction rules are not
implemented by this facade.

The duplicate compatibility state methods, dynamic-argument scanner, in-tag
comment loop, sequence matcher and entity-decoding algorithms are removed. The
published 35-variant `State` enum remains a compatibility vocabulary pinned by
the external consumer witness. Only the native 33-state machine executes; the
two unentered historical variants do not create runtime branches.

## Independent frozen evidence

Before applying the facade, the original machine and parser sources were
checked byte-for-byte against
`5bca3a881a011217738a79f000e9c8c03f21fc37`. The test-only baseline commit captures
the original output. Expected records were produced twice in fresh test
processes, compared byte-for-byte, then pinned before replaying the candidate.
The [capture receipt](./2026-10-01-l1-tokenizer-capture.json) records the original
source identities and SHA-256 of every expected fixture.

The five tests retain 10,767 individual expected records:

| Observation                                 | Cases | Coverage                                                                                                                                                     |
| ------------------------------------------- | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Every callback and live-mode poll           | 6,444 | 42 markup fixtures, complete inputs and every UTF-8 prefix/suffix, three switch combinations and repeated EOF                                                |
| Public switches and delimiters              |    14 | Empty/custom/arbitrary byte delimiters, raw custom-close behavior, Document declarations, entities, comments and callback-controlled verbatim mode           |
| Interrupted callback and resume             |     1 | A deliberately caught callback panic, retained non-EOF state, profile/raw/comment changes and later EOF switching                                            |
| Complete parser AST and ordered diagnostics | 4,296 | The same 42 fixtures and UTF-8 cuts under Component and Document parsing                                                                                     |
| Parser policy                               |    12 | Custom delimiters, Vue 1 raw interpolation, comments, namespaces and live `v-pre`, including modifiers/malformed heads and redundant implicit-close recovery |

Each 16-byte XXH3-128 record hashes little-endian u64 length-framed source and
all observed output fields. Callback records include complete ordered traces
and mode-poll counts; parser records include complete AST and diagnostic Debug
representations. Each failure identifies its profile, fixture and exact byte
cut and displays the actual fields. Counts and byte lengths also reject missing
or surplus records. Aggregate hashes remain an additional check. Raw original
observations and the original observer sources were preserved during capture;
the committed fixtures are bounded fingerprints, not the large raw corpus.

These records establish the enumerated cases, not all possible input behavior.
The old adapter-versus-tokenizer tests now compare two entries into the same
engine and establish internal consistency only. Existing independent compiler
snapshots, differential corpus and protected queue suites remain required.
Exact-head Actions must pass all 100 instruction ceilings without relaxation
before queue admission. Local targeted L1 and Armature laws, source length,
owned-storage inventory and module/dependency policies precede publication.

The first exact-head measurement at `c830edd84` exceeded two unchanged ceilings:
`armature_tokenize_small` used 4,081 instructions against 4,070, and
`armature_tokenize_stress-interp` used 202,147 against 200,943. All three runs
agreed; the other 98 probes passed. Removing an unentered enum variant had
shifted private `InRCDATA` from 33 to 32. The native enum now retains that numeric
gap without a dead variant or runtime arm. Its frequent Text/RCDATA transition
can use the original shift-and-add selection. A narrow x86 code-generation
diagnostic supports that mechanism; exact-head Actions determine whether all
100 actual benchmark ceilings pass. Frozen records and ceilings remain unchanged.
After rebasing onto `9c36db024`, three inherited embed-source test slices use
checked `get` access so strict Clippy can run without changing their assertions.

The gap-only exact head `2f9ebc4de` improved those probes to 4,077 and 202,045 but
still exceeded their ceilings. Interpolation now reuses `fast_forward_to` to
scan up to the closing delimiter's first byte. The skipped expression bytes
previously emitted no callbacks or mode polls. Empty closing delimiters retain
their behavior, and an unsuccessful scan leaves the same EOF cursor; matching
and partial-closing bytes still enter the existing close state with the same
state and delimiter cursor at every callback. All 10,767 frozen records and
strict Clippy pass locally; exact-head all-100 Actions remain required.

## Remaining scope

#6835 stays unfinished. The native component-surface constructor still admits
fixed delimiters and narrow options. Dialect-owned native `v-pre` scope control
must share the builder's namespace and implicit-close precedence; the facade
preserves the existing callback owner and does not implement that controller.
Native Document tree rules, custom/raw surface admission and full surface parity
remain work. Typed language embeds and Shape dispatch are tracked by #6836.
All-level capture, compiler, SSR and Vapor retain their existing product parser
routes and their separate admission gates. No fixture tolerance, instruction
budget, serialization boundary or pipeline stage changes in this slice.
