# Explicit release protocols and pre-1.0 minor versions

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6239](https://github.com/ubugeeei-prod/vize/issues/6239).

The public release guide described the default current-main atomic promotion,
but omitted the implemented immutable source protocol used by v0.436.0. Its
opening `patch` example also disagreed with the redundancy guide's rule that
every pre-1.0 release increments the minor version.

Document both existing protocols and lead with `minor`. The pinned path retains
its frozen source and source-run artifacts, delivers generated metadata through
the protected queue, authenticates actual signed delivery and catalog identity,
then lets the official runner tag the frozen source and publish its artifacts.
Resume uses the source PR with `--pin`; the source stays draft during
qualification. The default path retains its current-main refresh and atomic
main/tag transaction. These are operational descriptions of existing code;
source, permissions, required checks, tag identity and publishers are unchanged.

The guide requires separate evidence for protected metadata delivery, full
source checks, public assets, registries and installed issue reproductions.
Open VSX stays a separate channel. It grants no completed release, P0 closure,
n8n adoption, numerical performance result or upstream action.

Validation reads the actual public MoonBit CLI forwarding in
`tools/moon/cmd/release/main.mbt`, Rust start/watch/delivery implementations in
`tools/support/release/pr_pin_*.rs`, and the existing release workflow. Scoped
Markdown formatting and hosted ordinary docs/tooling checks qualify the change.

The v0.436.0 recovery exercised this boundary: after signed #8255 delivery,
Release run 37717489915 was paused with all 25 successful jobs and 41 build
artifacts intact. The frozen source PR retained its exact draft/body/head/pin
through a controlled close/reopen. The first event still used an older cached
merge snapshot and could not acquire the repaired native recipe. A second event
was admitted only after authenticating merge 4736bf9262 and its delivered recipe.
The stale source run was cancelled; its failure receipt remains preserved.
Publication stays pending until the fresh source and public requirements pass.
The [paired lifecycle receipt](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-6052770394)
retains the complete identities. This changes no release source or tag.
