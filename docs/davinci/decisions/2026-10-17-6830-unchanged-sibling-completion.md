# Preserve completion documentation across unchanged project synchronization

Full Check [38929265536](https://github.com/ubugeeei-prod/vize/actions/runs/38929265536)
at `a00fb6cff9268e1371ba260739d905050e2a2573` failed the existing real VS Code
host assertion that `ownSlots.header` resolves to Markdown. Artifact
12057958902 retains the authored sources and hover responses, but no
completion/resolve packet. The failure alone does not identify the cause.
The separate TS-45 four-editor qualification passed all 16 steps and their
negative controls; that scenario does not exercise this host operation.

A passive first-observation capture at `15cea8fa28f3b5cf595265e41ff690e5165532be`
passed the original host action in run 38931939784. All eight original member
completion operations returned Markdown; `header` was the sole candidate at
index zero. That observation rules out the first-16 resolution limit for
that run only. It is not a repair or a reconstruction of the failed response.

A deterministic native control preserves the complete original Rich, Slot,
and InvoiceCard sources, the exact incomplete Slot overlay, actual locked
Vue declarations, and native 7.0.2. It first materializes Rich, then adds Slot
to the same Canon project, issues the original header candidate once, and
interleaves one explicitly ordered diagnostic before resolving that candidate.
Run [38934300022](https://github.com/ubugeeei-prod/vize/actions/runs/38934300022)
at `7e13f5bb28520a9c643c6e7e515b7764c4cb95f6` actually executed nine tests:
immediate resolution, unchanged same-document resolution, and all six
existing Maestro resolve contracts passed; unchanged sibling diagnostics
caused the sole failure. Artifact 12059823868 preserves complete raw native
protocol and first-issued/resolved packets. Native SHA256 is
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
An earlier diagnostic source failed compilation before any test ran; it
provides no product evidence.

The failing native trace shows Slot materialization reloading the same
canonical project and closing unchanged Rich and InvoiceCard overlays.
After the candidate is issued, Rich diagnostics reopen those exact buffers.
That real mirror mutation correctly advances the global generation and
revokes all issued completion capabilities. Consequently the original
resolve is refused before any `completionItem/resolve` reaches native.
Neither authored document changes its bytes or revision in this case.

The repair changes only project synchronization. Before a topology reload,
retain an existing overlay only when the root and effective configuration
are unchanged, the root is Canon's generated project, the URI names the
same canonical physical file, disk bytes equal the retained bytes, and the
file is absent from every changed/created/deleted set. Restore those buffers
after the existing reload so subsequent sibling diagnostics need no close
and reopen. Actual materialized-file notifications still advance generation.
Root, effective configuration selection, and materialized-session transitions
retain nothing. Editing the selected configuration at the same path still
reloads the project and forwards its watched-file revision, invalidating
issued candidates; unchanged canonical buffers may remain open.
All completion generation, whole-issued-item, foreign-position, reopen,
mutation, and native-session refusal code remains unchanged. There is no
resolve retry, new completion query, longer deadline, or relaxed assertion.

The legacy corpus at `tests/_fixtures/differential/lsp/unchanged-sibling-completion`
retains complete authored inputs and both successful and failed historical
packets. Additional native controls edit a sibling, delete a dependency, and
reopen the target with identical bytes/version; each requires the original
candidate to remain wholly unresolved. Canon controls cover unchanged byte
and path identity, all materialized delta kinds, missing files, symlinks,
root/configuration/mode transitions, and the existing six issued-item laws.

Repaired source `80f6dc4c86e5f8d8c4f9eb5f0134f3281c18345d` passed all 21
focused tests in native job 117094078961 of
[run 38936756145](https://github.com/ubugeeei-prod/vize/actions/runs/38936756145):
three Canon retention laws, six unchanged Canon issued-item laws, and twelve
Maestro cases including the six real-provider controls. No selected test was
ignored; the twelve Maestro cases took 4.01 seconds. Artifact 12061671753
has SHA256 `69b531a3d106f288c91a13a890bb319ced22d5a381b87c1d1a82449fd0ef9d07`.
Its native executable hash matches the failed historical control. The repaired
sibling trace contains one completion, unchanged Rich diagnostics, and exactly
one native resolve returning the full signature and Markdown. The unnecessary
Rich/Invoice close and reopen are absent. All three negative cases issue one
completion and zero native resolves, preserving the entire original item.

The dependent unchanged host job 117094929439 failed before building the CLI
or starting an editor. Downloading the pinned 167,389,328-byte Zed extension
CLI timed out twice at the existing 90-second limit: 150,142,904 bytes and
100,351,595 bytes received, then curl exit 28. No host completion packet was
produced. This is not host qualification and not an observed product failure;
the provider, checksum, download limits, and assertions remain unchanged.

Permanent coverage adds the same prepared six-case execution to the existing
scheduled/full editor host action after its frozen dependency install. It
reuses the original provider preparer and keeps native packets in a separate
small artifact. There is no new daily lane or dependency installer. The
landed host observer preserves only the first public completion response,
selected index, source, and actual Markdown fields; diagnostic follow-up
completion/resolve probes are excluded. Both new Rust test companions use
the sibling `_tests.rs` layout.

The temporary diagnostic workflow is excluded from the landing manifest.
Fresh final combined-source Actions and actual host execution, protected
merge, and release remain separate requirements. The original unrecorded
host failure must not be retroactively asserted to have this exact causal
chain.

The first combined PR run `38986992717` passed all Rust cohorts but correctly
failed feature-lane Clippy on this test companion. Clippy recognizes the
workspace's assertion exemptions through a plain `#[cfg(test)]` module,
not a combined test/feature/platform predicate. Keep that attribute separate
from the unchanged native/Unix condition, matching existing test modules.
Unknown fixture interventions still fail explicitly with their name. No lint
is suppressed and no product or fixture assertion is removed; fresh Actions
must verify the complete feature lane again.

Full Check [38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
at `e2b883eed6378eb7ada73bf0220d89a185a3d6fd` passed the physical editor host
in 262 seconds. The original `ownSlots.header` and `$slots.header` operations,
complete Markdown and TypeScript fence, Header text, and exact same-URI
navigation assertions passed. The retained native completion packets also
contain complete Header Markdown for immediate, same-document and sibling
resolution. Feature Clippy passed. Other docs, tooling and performance jobs
failed in this Full Check, so this establishes these host contracts only;
fresh combined-source qualification is still required.
