# Literal original Oxlint ignore replay

Issue: [#7903](https://github.com/ubugeeei-prod/vize/issues/7903).

The discovered-configuration fix #8167 actually merged as signed commit
`261da112d7954b737dcc666a99bb89e698248609`. All exact protected workflows passed,
including [Check 37629433260](https://github.com/ubugeeei-prod/vize/actions/runs/37629433260).
The existing scope fixture adds a debugger statement and different filenames
to exercise independent core and override controls. Keep it unchanged, but
do not substitute those controls for the complete original reproduction.

Add the original issue body, literal `printf` SFC source, heredoc configuration,
VCS ignore bytes and original four paths as a separate frozen corpus. Preserve
the package specifier by binding this source checkout's packed plugin through
the fixture's ordinary `node_modules` directory. No input/configuration
projection changes the authored original bytes.

The independent expected wrapper packet uses the existing source-authored
#7904 no-v-html message and the identical original SFC range, changing only
the original #7903 filename. Do not capture product output to author expected
diagnostics. Compare every foreign/core diagnostic field with stock Oxlint.
Require only `src/AppPanel.vue` to be selected and reported; all VCS, config
and CLI exclusions must stay absent. Retain every complete default and JSON
process packet before asserting, plus source/config/ignore bytes before and
after each process and cleanup of all owned temporary paths.

Independent review strengthens failure custody: save each whole raw process
before reading post-call inputs, so deletion cannot discard the failed packet.
Give each child a private temporary directory beside the original project,
outside its lint/config roots, and compare the complete recursive owned tree
before and after every call. Global temporary-directory activity from concurrent
work never supplies or invalidates this cleanup proof.

Run the literal original command and its JSON report three times on stock and
wrapper paths, for 12 complete observations per host. Reuse the existing
source-built Actions qualification and actual Oxlint 1.78/1.86 host installations;
both archives remain under the already uploaded original-project artifact.
No new pipeline stage, install path, deadline, instruction ceiling or historical
corpus change is needed. A local binding does not supply compiled acceptance.

This bounded replay does not close #7903 by itself. The original issue also
mentions standalone HTML. The current HTML-only adapter creates temporary
copies before ignore selection; mixed Vue/HTML discovery conservatively
refuses transport, even when the HTML candidate is ignored. Stock Oxlint does
not select standalone HTML, so its selected file set cannot qualify that
separate adapter's ignore semantics. TODO: preserve the adapter's supported
positive behavior and original ignore rules in an independently proved HTML
selection contract. Nested configuration, JSONC and installed package replay
also remain separately unqualified. n8n adoption #8142 stays open, and upstream
n8n remains read-only.

Exact-head source Actions, protected full qualification and actual merge are
required before crediting this new original reproduction replay. Root owns
queue admission and installed publication; stale green #8167 checks do not
qualify the new observer.
