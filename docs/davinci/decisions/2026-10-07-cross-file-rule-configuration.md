# Cross-file CLI rule configuration

Issue: [#7935](https://github.com/ubugeeei-prod/vize/issues/7935).

The CLI's project passes currently bypass the resolved per-file linter config.
Pass their new diagnostics through the existing resolved rule map before merging
into ordinary lint results. This retains declarations and entry matching exactly
as resolved for single-file lint, without another config load or source parse.

Accept the four existing route-typing IDs, the `cross-file` group and the existing
60 published Croquis codes in CLI config validation. Keep single-file native
metadata unchanged because its consumers export every listed rule to Oxlint,
which does not run project passes. An explicit individual code overrides its
group; an explicit rule/group overrides category warning/error. Category off keeps the ordinary disabled-category precedence.
Route findings follow `ecosystem` and the existing `suspicious` category; Croquis
findings follow `cross-file`. Do not reclassify every Croquis finding into a
coarse category. Keep default diagnostic IDs, messages, spans and defaults.

The fixture retains the whole reported issue/config and authored direct-router
controls for all four route diagnostics plus an unmatched inject. Real CLI tests
cover rule off/warn/error, full JSON field conservation, group and individual
code controls, category severity/disable, ordered entry resolution and exit
status. Independently authored equal-severity inject rows follow the existing
message tie-break order; default mixed-severity ordering remains intact. Regenerate
the observational consumer inventory for the new imports. Exact-head Actions,
protected merge qualification and released inclusion remain required; local source preparation is not execution evidence.

Existing merged #8070 and #8091 already repair script browser positions and deep/
slotted CSS subjects. Their incomplete public/original invocation proofs and the
separate global-selector proposal are tracked independently; this change does
not claim those issues completed.

## Duplicate consolidation

The read-only comparison selects [#8155](https://github.com/ubugeeei-prod/vize/pull/8155)
as the canonical #7935 delivery and closes [#8173](https://github.com/ubugeeei-prod/vize/pull/8173)
as a duplicate. Keep the #8173 branch and its historical source
`3bc30d627809f9a1d9e6b52abb8ce137098d5d7f`: the three compiled CLI tests retain
19 complete JSON invocations, authored mixed/equal-severity references and the
original issue/config. Its [source Check](https://github.com/ubugeeei-prod/vize/actions/runs/37591201182)
passes all source/Rust/JS/tooling/browser checks while the old shared SDK audit
and dependent report fail; its [native backend run](https://github.com/ubugeeei-prod/vize/actions/runs/37591200703)
passes. These results qualify that historical head, not this docs-only cleanup.

#8155 configures structured producer diagnostics before rendering, covers all
four route rules plus Croquis, composed HTML and attrs fallthrough, and retains
74 authored cases with 148 complete JSON/plain process contracts and 252 JSON
file rows. Its canonical normalized `croquis/cf/*` IDs, `cross-file` project group
and existing `suspicious` category supersede this draft's full `vize:` config
spelling and new categories; do not combine the two policy implementations.
The earlier configuration discussion above remains historical preparation.

The original #8155 source `ae0642ff1389fc18da9b68e348f657bad196f479` has passing
source/native checks and an official artifact whose complete process bytes,
argv, status and before/after inputs match all 148 authored references. Current
canonical source `c90da23d45171c1117a285ebc4a4e07440d75086` incorporates actual
SDK main `706a5b7886c363f6c67a03964ac55f26c5a2a341` with unchanged production,
test and corpus bytes. Its [fresh source Check](https://github.com/ubugeeei-prod/vize/actions/runs/37594059054)
and [fresh native run](https://github.com/ubugeeei-prod/vize/actions/runs/37594058072)
remain pending at this decision. Keep #7935 open until canonical actual merge
and acceptance; no historical green transfer, queue admission or release
completion is claimed by closing the duplicate.
