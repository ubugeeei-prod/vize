# Census infrastructure source checks

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed unnecessary workspace selection

The original source `795e3d312e68f45bbba0e03b9dc4bc0be359f181` of
[#7580](https://github.com/ubugeeei-prod/vize/pull/7580) changed census report
infrastructure and generated consumption Markdown, with no Rust package source
change. Hosted [Check 37121764436](https://github.com/ubugeeei-prod/vize/actions/runs/37121764436)
nevertheless selected the entire metadata workspace: `changedPackages` was empty,
while the generated reports and three census JavaScript files triggered the
shared/unknown Rust fallback. The Rust producer took 2m52s; the four test workers
and Rust report made that lane's critical path 5m24s. The same source selected
561 of 712 tooling files. These are observations of that exact source campaign,
not a controlled performance comparison or proof of a two-minute whole gate.

## Bounded input classification

Only these Node-only census implementation files enter the tooling source lane
without selecting workspace Rust, JS package builds or playground tests:

- `tools/support/compat/davinci/croquis-consumers.mjs`
- `tools/support/compat/davinci/lib/croquis-render.mjs`
- `tools/support/compat/davinci/lib/croquis-shards.mjs`

The same classification applies to the exact generated index
`docs/davinci/plan/croquis-consumption.md` and its reserved direct
`vize[_<crate-part>].md` shard family under `croquis-consumption/`. The mandatory
level inventory action still checks the entire generated set byte for byte,
including orphan rejection and every actual consumer/product gate. No general
JavaScript or Markdown extension exemption is introduced. Unknown helpers,
authored contracts, nested files and other plan inputs retain conservative
selection.

A source-grounded tooling law discovers every current workspace member from
the root manifest and rejects named census renderer/report references in its
Rust sources. New member syntax or a direct consumer therefore requires reviewing
the classification. Mixed census and actual Rust changes retain the
same owner and complete reverse dependency closure, including optional, renamed,
target-specific, build and dev edges. Invalid paths, shared configuration,
compiled plan inputs, workflow changes and genuine native capture action changes
keep their existing source lanes. The affected native setup capture selector is
unchanged.

## Acceptance and limits

Merge groups unconditionally validate every current workspace crate, all four
full Rust test partitions, all four complete tooling partitions, the original
differential corpora and all 100 pinned instruction probes. The four source
lanes, Rust/source/final required report contexts, failure semantics and numeric
budgets are unchanged. PR tooling still uses its existing conservative selector
and source-built runtime preparation; this change does not claim that tooling or
the whole source gate takes two minutes.

Focused classification laws cover census-only selection, mixed real Rust and
native/config/workflow inputs, reserved-path near misses, actual source consumer
custody and unconditional protected selection. Existing hosted planner tests,
exact-head Actions and actual protected merge determine acceptance. This slice
has no implementation dependency on #7580; its independent queue admission follows
that canonical conflict repair's priority. No local Cargo build, installation or
extra manual full campaign is required.
