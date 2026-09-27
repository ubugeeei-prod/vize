# Private Patina S2 first-row capacity preparation

This candidate starts at frozen source
`444ff3460aec443bbe9e83f700e3c142d5e7c1f0`. The root approved this bounded
implementation direction privately. Root whole-patch review is required before
composition. There is no push, workflow dispatch, public issue comment or PR.
The accompanying publication draft supplies the paired issue and central-record
text for the integration owner; neither has been published by this child.

## Observed production work

The prior Entry change is actually measured: Patina S2 changed from 543205 to
542626 instructions, saving 579 against the unchanged 539107 cap. It still
exceeds that cap by 3519. Function self counts moved with compilation, so they
are observations rather than isolated algorithm savings.

For current 444, the actual Facts name-ID map has seven `reserve_rehash` calls:
16409 self instructions, 1969 in its allocation edge and 254 in its free edge,
18632 inclusive instructions. The parse block records the merged names/cased
Vec grow symbol in two separate edges: seven calls / 3341 instructions and one
call / 230 instructions. The data contains 146 distinct identities and one
ASCII-cased name, `svg:foreignObject`. Those edges fit seven names growths and
one cased growth, but the symbol alone does not establish exclusive type
attribution. The cased Vec is unchanged by this patch.

Read-only evidence is
`/tmp/vize-native6962-final-444-exclusive-attribution.json`, key
`patina_s2_markup_one_root`, and raw
`/tmp/vize-native6962-final-444-instruction-36326522402/run-1/vize_patina-davinci_markup.callgrind.2`.
All three current runs have matching measured inputs, method and ceilings.
Raw evidence, drivers, benchmarks, caps, workflows and input bytes are unchanged.

## Bounded capacity hint

Immediately after the first row successfully splits into exactly four tab
columns, count its direct member tokens, excluding `#text` and `@` references,
and stop at 256. Reserve that count once in the existing names Vec and ID map.
Continue the original streaming loader, Entry insertion, namespaces, case
handling, ID order and 256-element rejection behavior.

The committed first row happens to contain 92 distinct direct names; 92 and 146
are evidence, never production constants. Installed standard-library growth
rules suggest a names capacity of 92 then 184, compared with the current 256,
and a map capacity of 112 then 224, compared with the current 224. These are
source-derived expectations, not new runtime measurements or memory guarantees.
There is one additional scan of the first valid row's member field plus the
once-per-row boolean check. Whether fewer growths outweigh that work is unknown.

Empty, comment-only and malformed-only tables never reach the reserve block.
A first valid row containing only text/category-reference tokens has a zero
hint; a children parent can still allocate its original identity afterward.
Malformed rows cannot seed the hint. Unknown row kinds are deliberately handled
in the original order: members are parsed before row-name validation, so their
original identities and defects are preserved.

Duplicates and invalid conditions may overestimate useful identities. Even an
invalid or unknown first row can cause bounded reservation; there is no promise
of identical memory usage for small, duplicate-heavy or invalid tables. The
hint never exceeds 256 and is not a logical universe-size change. Existing std
Entry can still reserve for a rejected vacant key before the insertion guard.
Allocation counts, capacities, memory and numeric savings remain UNKNOWN.

No global initialization, memoization, prewarming, extra pipeline stage,
serialization, source-layout trial, dependency or hasher change is introduced.

## Complete authored oracles

The prior five controls remain byte-exact in `facts/load/tests.rs`. Four new
controls compare ordered names, the complete ID map, ordered cased names,
every row's bits/conditions/text/anchor, the whole children map and empty row,
plus the complete ordered defect vector:

- Empty, comment-only and malformed-only tables, then malformed prefixes
  followed by a real HTML/SVG/MathML children row.
- A first valid row with zero direct tokens, including a forward category
  reference and unknown row kind, followed by a normal children row.
- Duplicate and invalid-condition tokens in an unknown row before validation,
  followed by conditional SVG membership and exact namespace/ID order.
- An oversized first unknown row that fills exactly 256 identities, rejects
  SVG/MathML vacancies and a children parent, then preserves every occupied hit
  and the entire logical state after further rejected insertions.

The expected 27 missing-row defects and all row-state fields are complete
authored artifacts, rather than prefix/substring probes or capacity assertions.
Generated fixture names exist only inside the new test; committed inputs and
archived outputs are unchanged.

## Preparation and remaining work

Local preparation uses only Rustfmt, the existing JavaScript assertion scanner
with both cfg(test) files explicitly treated as whole test files, source-cap
checks, diff checks and read-only metadata/byte comparisons. The loader is 188
lines and new test file is 265 lines, below the 350-line cap. These checks do
not establish Rust type correctness or runtime behavior. No local Cargo command,
product build, install, runtime test or measurement is performed.

Byte proof reverses only the new test-module declaration, capacity boolean and
bounded reserve block to recover the complete original loader. Existing five
controls, `facts.rs`, the WHATWG table and all other tracked paths outside the
four-file candidate are protected unchanged.

TODO: root review this entire patch before composition, then publish the paired
decision draft with the central record in the same integrating change. Actions
must type-check and run all nine authored controls plus existing Patina tests.
The unchanged exact-head native gate must measure all 100 targets and allocation
controls. The cap must pass without changing the input, method or ceiling.
Until then runtime correctness, net savings and allocation identity are UNKNOWN.
