# Per-Art VRT report ownership

Issue [#8479](https://github.com/ubugeeei-prod/vize/issues/8479), paired with
[the ownership decision](https://github.com/ubugeeei-prod/vize/issues/8479#issuecomment-6095862683).

The real gallery API writes both `left/Button.art.vue` and
`right/Button.art.vue` to `vrt-Button-report.json` and `.html`. The second
Art overwrites the first complete report despite distinct native PNG owners.
The byte-exact blue Left and red Right Art fixtures remain outside the
differential compiler collector; a JSON history fixture authenticates them.

Use the existing portable project-relative Art identity. Retain ordinary safe
basename report names when the complete discovered Art set has no
case-insensitive collision. Qualify colliding or unsafe names with a full SHA
of the portable identity. Reserve that qualified namespace so an authored
basename cannot impersonate another report owner.

Before any capture or report write, inspect existing report ownership. Refuse
foreign owners, malformed JSON, orphan HTML, nonregular files and case-fold
conflicts with an explicit archive/move instruction. Adopt a legacy JSON only
when every existing result belongs to the selected Art. Add portable owner
metadata only to per-Art API JSON. Ambiguous old basename reports remain
untouched. Shared CLI JSON/HTML generators and all-Art report names remain
unchanged. Removing a colliding Art cannot authorize overwriting a foreign
legacy report.

Require the actual source-built native compiler, Vite gallery, browser Run VRT
button, real API responses and physical screenshot runner. Retain each complete
JSON/HTML/PNG before the next capture. Assert both native PNG owners differ,
the second capture preserves the first whole report, repeated comparisons pass
and preserve the other whole report. Fresh exact-head Actions, protected actual
merge and publication remain pending. Hosted browser VRT needs a separate
Node screenshot service and remains unfinished.
