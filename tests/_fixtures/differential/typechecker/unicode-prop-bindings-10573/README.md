# Unicode prop binding projections

Issue #10573 reports an optional `café` prop whose generated declaration was
`const caf_ = props["café"]`, followed by a synthetic `void café` reference.
The real mapper host reported an unmapped TS2304 while the CLI suppressed that
generated-coordinate diagnostic. A `café`/`caf_` pair also emitted duplicate
`caf_` locals. The prop declaration emitter now shares the OXC lexical identifier
authority already used by setup anchors, while retaining the existing alias
policy for reserved and nonidentifier property names.

The eleven authored cases cover the report, Unicode/ASCII collisions and their
distinct value types, quoted and escaped cooked keys, a supplementary-plane
identifier with CRLF, a Japanese runtime prop, a defaulted prop, a keyed type,
reserved/hyphenated/ASCII controls, and a Unicode continuation character.
`edit-cycles.json` pins complete expected mapper messages and CLI diagnostic strings.
The CLI and real pinned mapper host each run every valid, invalid and repaired
source. No finding is filtered, deduplicated or rewritten during comparison.

The native projection backend also records every complete checker projection
and checks all emitted mapping and semantic-link endpoints against their source
and generated UTF-8 boundaries. The real CLI and official host tests establish
semantic acceptance/rejection; the projection backend alone does not run a full
TypeScript type engine. `analysis_time_ms` is omitted from projection custody.

The product differential registration independently pins the complete valid
authored input for each case. The dedicated workflow preserves all raw CLI and
mapper outputs and both provider identities, and compares all 228 unaffected
checker/mapper packets against the exact PR base without changing their oracle.
Each role starts with an absent, separate compiled target. Actual Cargo JSON,
fresh Canon/observer artifacts, Git-verified source bytes and executed observer
paths/hashes are retained; compiled targets are never shared between roles.

Before-fix physical `@vizejs/native@0.566.0` API results for all eleven cases are
retained alongside the reported public receipt. They confirm the actual emitted
bindings, including duplicate `caf_`, `_value`, `___`, and `a_b` aliases. Source
qualification and fresh publication acceptance are separate delivery gates.
