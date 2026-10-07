# Art variant setup reads

Issues: [#7940](https://github.com/ubugeeei-prod/vize/issues/7940) and
[#7900](https://github.com/ubugeeei-prod/vize/issues/7900).

The original registration correction in #7964 is already merged. The remaining
unused-binding rule treats Art files as script-only SFCs and reports before any
variant can read their shared setup bindings. The same absent value map makes
the SSR rule classify a local `close` handler as a browser global.

Keep the genuine descriptor scripts and their original definition spans. Seed
each existing variant Drawer with only their value binding metadata, definition
spans, and demanded unused candidate relation. Every variant retains its own
fresh template scopes and facts; the original borrowed script summary remains
the registration authority. The original fragment is parsed and drawn once.
No whole Croquis clone or sibling template-scope reuse is required.

Intersect the remaining unused candidates after each variant. Report the shared
physical script once after all variants, using the existing script frame,
configuration and suppression handling. An ordinary template in the same Art
file contributes its reads to that conclusion. Existing style reads, component
and directive spelling, refs, `v-for` sources and conservative unknown/eval
refusals remain in force. A local value named `close` is now visible to the SSR
rule; genuine browser globals remain findings in each original variant.

The differential corpus in
`tests/_fixtures/differential/linter/art-setup-bindings-7940/` preserves #7940's
original SFC and #7900's original SSR SFC, hashes every input, and authors twenty
complete service results. Existing #7897/#7900 originals and the eleven
registration laws remain unchanged. Controls cover all-variant unions, exactly
one physical unused span, dual scripts/UTF-8/CRLF, local `v-for`/slot shadows,
component/directive/ref reads, style and ordinary-template reads, eval/external
style refusals, script suppression, empty Art and ordinary Vue behavior.

Rust formatting and corpus source checks are local preparation. Compilation,
the authored whole-result controls and existing registration laws must execute
on the PR's exact source in Actions, followed by protected queue suites and
actual merge. Release and installed-package evidence are separate. This change
does not complete native/default migration or the lint fix-history gate.

The first source `8b5cb076` was rejected by Check 37589451607/job 112687306804
for one owned Clippy warning: the final `Option<&mut Croquis>` dereference was
redundant. Consume that existing option directly; no relation, original input
or expected whole result changes. Fresh successor Actions remain required.
