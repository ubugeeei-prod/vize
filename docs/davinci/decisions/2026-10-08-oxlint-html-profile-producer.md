# Internal original HTML selection producer

Issue: [#7903](https://github.com/ubugeeei-prod/vize/issues/7903).

Build one bounded internal selection producer beside the existing native lint
file collector. It qualifies original-path HTML selection primitives before
wrapper integration. It does **not** implement the `oxlint-vize` front-end,
qualify standalone/mixed HTML output, satisfy public-installed requirements or
complete n8n adoption. #7903 remains open.

## Ownership and provider identity

The current wrapper collects original files in JavaScript and creates carriers
before Oxlint invokes the first native `lintPatinaSfc(source, options)` call.
That supplied-source call has no target discovery ownership. Adding selection
to its return would be too late to exclude originals before carrier creation.

The existing native `lint(patterns, options)` call owns file collection before
actual linting. Its current defaults are not the pinned Oxlint contract, and
the wrapper does not call it. This slice adds a private, independently tested
profile producer alongside that ownership. Existing default collection and
public NAPI functions/options/results remain unchanged. The profile is not
connected to a public or wrapper invocation in this slice.

Use the existing locked `ignore = 0.4.33` dependency. Preserve the actual
`WalkBuilder`, `GitignoreBuilder`, `IncrementalIgnore` and `OverrideBuilder`
matching operations and their phase ordering. Git subprocess selection, an
approximate conjunction of predicates, synthetic extension probes and stock's
unsupported HTML absence are not the selected architecture. Add no new native
query, pipeline stage, OXC dependency family update or serialized level boundary.

The inspected host profiles are:

| Host   | Exact source identity                      | VCS root contract                                                        |
| ------ | ------------------------------------------ | ------------------------------------------------------------------------ |
| 1.78.0 | `c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd` | Gitignored explicit files and directory roots are excluded.              |
| 1.86.0 | `2ae2939bb2fd98796393658b21556b2a2467e047` | Explicit files remain eligible; gitignored directory roots are excluded. |

Both keep VCS filtering under `--no-ignore`. That flag disables the custom
ignore-file and CLI override sources, not configuration ignores. Walk defaults
include hidden files, disable generic/global ignore sources, retain parent and
VCS/info-exclude matching, follow links and prune VCS metadata. Qualify only the
explicit finite envelope below; these defaults do not grant untested filesystem
or configuration acceptance. [Selection order][lint-178], [1.86][lint-186];
[shared traversal][walk-178], [1.86][walk-186].

## Bounded producer contract

The initial boundary carries the original cwd, one literal existing file or
directory target, the exact host profile and one explicit regular root JSON
selection envelope. The producer parses the original JSON object and
`ignorePatterns`, retaining inheritance/override refusal controls; it does not
validate arbitrary rule, plugin or settings fields. Complete engine configuration
validation and its original error/report contract remain successor requirements.
Retain target provenance and original path spelling;
return a deterministic sorted selected-set/provenance record. Stock's parallel
walk does not promise that order, so this grants no whole-report ordering credit.
The returned original set contains genuine `.html` and `.htm` files only;
supported-source selection and complete stock reports remain successor
responsibilities. No renamed input or transport carrier participates in selection.

Apply original custom-ignore/CLI prefiltering, profile-specific VCS root
filtering, original traversal/override matching, then root configuration ignores
in the actual host order. Configuration whitelists use
`matched_path_or_any_parents`; their full-file whitelist behavior is not Git
ancestor pruning. During walking, a custom-ignore whitelist can outrank VCS;
the explicit-file custom and CLI prefilters independently exclude roots before
the VCS check. An override result does not cancel a separate custom root
exclusion. CLI patterns are mechanically reversed: a user leading `!` becomes
a literal filename character in an exclusion, rather than a re-inclusion.
Qualify these separate rules with whole controls.

Unsupported envelopes fail explicitly rather than returning an apparently clean
selection: globs, multiple roots, nested or inherited configuration, unresolved
config authority, suppression state and unqualified filesystem metadata cannot
receive incidental acceptance. The precise supported envelope and every refusal
must be visible in the typed producer contract and its independent fixtures.
Whole wrapper report retention is a future integration responsibility, not an
internal selected-set claim.

Preserve every older literal original Vue/HTML input, configuration/ignore byte,
argv, authored report, observation count and instruction/batching limit. Add new
independently authored genuine HTML controls and whole selected-set/provenance
expectations; never replace the existing literal producer with this new one.
Record source/config/ignore authority, setup and complete recursive owned-tree
custody. Failure controls preserve the original failure evidence.

## Qualification and successor

The separately authored corpus contains 45 complete cases: 21 selected-or-empty
sets and 24 explicit refusals, each scheduled once on both pinned profiles for
90 intended observations. Its frozen `cases.json` SHA-256 is
`e23dbe7b93ba5ebb4be30e2bd7c1a6494d4f8fd511ab5df8a944f023dee8164f`.
Independent pre-execution review preserves the first 33 whole case objects and
all 118 older tracked input blobs, apart from the private module declaration;
the existing 3,496-byte collector and tests body remains byte-exact. These counts
and hashes record prepared controls, not successful runtime observations.

Actual source-native Actions must compile and execute the internal producer's
whole fixtures on both exact host profiles. Save the complete selected paths,
provenance and refusal/error evidence; no stock HTML absence or unrelated Vue
positive can qualify a genuine HTML result. Existing differential suites and
all instruction ceilings remain enforced. Local source or registry binaries
cannot substitute for hosted source identity.

Initial source `b74f35bd` failed Rust test compilation before any producer
observation: `sha2` 0.11's digest array does not implement `LowerHex`. Render each
byte with its scalar hexadecimal formatter and remove the unused test import.
Preserve all 45 authored cases, source/config/ignore bytes, full expected sets,
counts and ceilings. The failed source grants no runtime acceptance; the
successor requires fresh compiled Actions and all 90 actual observations.

Source qualification, protected queue validation and actual merge are separate
steps. Until those are terminal, this is prepared internal work. Even after
merge, the wrapper and installed acceptance stay unqualified.

A dependent successor must review a minimal replacement of the HTML producer
inside the existing collection-and-lint ownership. It must return selected
originals/provenance and actual HTML diagnostics in the same call, eliminating
HTML carrier linting rather than adding a selector query or duplicate lint pass.
Keep stock authoritative for supported-source core/custom reports; retain whole
formats, order, totals, exits, options/settings and write refusals. Public Rust
API compatibility needs an explicit design; casually adding fields to published
structs is insufficient. Register dependent PRs as a GitHub native Stack and
admit only a contiguous exact-head-green prefix through the protected queue.

Root owns first canonical clauses, queue admission and official publication.
Pair this decision on #7903 and in the canonical record before commit. No
upstream n8n/OXC comments, changes or publication are authorized.

[lint-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/apps/oxlint/src/lint.rs
[lint-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/apps/oxlint/src/lint.rs
[walk-178]: https://github.com/oxc-project/oxc/blob/c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd/crates/oxc_config/src/walk.rs
[walk-186]: https://github.com/oxc-project/oxc/blob/2ae2939bb2fd98796393658b21556b2a2467e047/crates/oxc_config/src/walk.rs
