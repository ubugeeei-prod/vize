# Private paired publication draft: Patina Facts capacity

This file is a draft for the integration owner. It is not an issue comment,
central-record update, publication or claim of passing runtime validation.
Root must review the whole patch before composition and bind both texts below
to the exact composed SHA and measured evidence before publishing them together.

## Issue comment draft

Patina S2's actual Entry candidate measures 542626 instructions against the
unchanged 539107 cap, still 3519 over. The follow-on private candidate reserves
the existing Facts names/ID containers once from the first valid four-column
row's direct member count, excluding text/category-reference tokens and capped
at the existing 256-element universe. This addresses seven observed ID-map
rehashes and early Vec growth without a fixed table size, memoization,
prewarming, input changes or cap changes.

The original streaming parse order, exact namespace/name keys, ID/case order,
256 overflow behavior and defect ordering remain the intended contract. The
prior five controls are unchanged; four additional complete-state/defect
oracles cover malformed prefixes, zero direct members, duplicate/invalid tokens,
unknown row kinds and first-row overflow with occupied hits at capacity.

The reserve hint can overestimate duplicate/invalid rows. Prescan overhead,
allocation/capacity identity, Rust type/runtime correctness and net instruction
savings are unverified. Local preparation performs formatting, whole-assertion,
source-cap and protected-byte checks only. No local Cargo/product build is run.
TODO: whole-patch root review, exact-head Actions tests, then all 100 native
targets and allocation controls under the unchanged measurement contract.

## Central decision-record draft

Decision: use a single first-valid-four-column-row capacity estimate for Facts'
existing names Vec and ID map. Count only direct member tokens, excluding
`#text` and `@` references, with a maximum hint of 256. Malformed rows do not
seed; a zero direct-member first row reserves zero. Parsing members before
unknown-row validation remains unchanged, preserving logical state and defects.
No fixed committed-table size, source-layout change or new cached initialization
is introduced.

Evidence: actual source 444 measures Patina S2 at 542626 / cap539107; its name-ID
map performs seven rehashes, 18632 inclusive instructions. Fewer early growths
are a hypothesis. Duplicate/invalid hints can overreserve; all allocation and
numeric acceptance remains dependent on exact-head measurement. The paired
companion is `2026-09-27-patina-facts-first-row-capacity.md`.

TODO: preserve the five original complete controls and run the four new complete
logical-state/defect controls in Actions. Record the exact integrating SHA,
all 100 native target results and allocation results, preserving raw inputs,
drivers, method and ceilings. Do not call S2 complete or passing before that
evidence exists.
