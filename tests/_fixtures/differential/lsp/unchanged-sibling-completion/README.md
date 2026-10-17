# Completion documentation after an unchanged sibling diagnostic

The three `.vue.txt` files are complete, unchanged sources from the real
VS Code host fixture at `a00fb6cff9268e1371ba260739d905050e2a2573`.
Their original strict Bundler configuration is retained. The existing real
workspace preparer enables TypeScript extension imports and attaches the
locked Vue package and the extension's actual TypeScript native runtime.
The test compares every authored source byte against this archive.

Open RichAuthoring at version 5 and SlotAuthoring at version 6, with the first
`ownSlots.header` changed to the incomplete prefix `ownSlots.`. Collect Rich
diagnostics, then Slot diagnostics, and request completion at that prefix.
Resolve that exact returned `header` item once, either immediately, after
unchanged Slot diagnostics, or after unchanged Rich diagnostics. Each case
uses a fresh workspace and a single native session. No resolve retry, new
completion query, synthetic native answer, or modified expectation is allowed.

At source `7e13f5bb28520a9c643c6e7e515b7764c4cb95f6`, real native 7.0.2 passed
the immediate and same-document cases and failed the sibling case. Actions
run [38934300022](https://github.com/ubugeeei-prod/vize/actions/runs/38934300022),
job 117085779070, artifact 12059823868 retain the complete protocol exchanges.
`historical.json` preserves all three complete issued and resolved packets,
source revisions, source hashes, native executable identity, and provenance.
The failed packet is intentionally retained without documentation.

The successful oracle is the complete authored header method signature and
all original JSDoc, including Markdown, parameter text, and the example.
The entire insertion packet must remain unchanged. Derived negative controls
edit Rich's parameter type, delete its InvoiceCard dependency, or reopen Slot
with identical text and client version. Each must refuse the original whole
candidate. Separate Canon laws cover edited/created/deleted/missing files,
physical path identity, project/configuration-selection/mode transitions, and all
existing issued-candidate/native-session refusals.

This deterministic native result establishes one cause of missing completion
documentation. The original failing full-host run did not capture its actual
completion packet, so it does not prove that run followed the same path.
