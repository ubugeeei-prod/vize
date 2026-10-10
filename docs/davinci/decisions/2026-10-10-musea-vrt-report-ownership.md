# Per-Art VRT report ownership

Issue [#8479](https://github.com/ubugeeei-prod/vize/issues/8479), paired with
[the ownership decision](https://github.com/ubugeeei-prod/vize/issues/8479#issuecomment-6095862683).

The real gallery API writes both `left/Button.art.vue` and
`right/Button.art.vue` to `vrt-Button-report.json` and `.html`. The second
Art overwrites the first complete report despite distinct native PNG owners.
The byte-exact blue Left and red Right Art fixtures remain outside the
differential compiler collector; a JSON history fixture authenticates them.

Actual native [Actions 38039504300](https://github.com/ubugeeei-prod/vize/actions/runs/38039504300)
at `db37087a0984461a3e191fd63e29a4cad675def5` failed only the report-path
inequality; original native inline/globals, project-options and Chromium
controls passed. Complete physical Left and Right JSON/HTML/PNG were retained.
Both report paths were identical, and final physical JSON/HTML were exactly
Right's retained bytes. The distinct PNG SHA256 values were
`376028536bd4122df82cb30bd85aed4fc0aea24661237df77a77c6d67372579f`
and `c7c74a69d3d9d530ddcaa7bf5d8a185da929a3270a06c74db9a9f9cdc3990564`.
This authentic old-source failure grants no successor qualification.

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

Actual successor [Actions 38040050699](https://github.com/ubugeeei-prod/vize/actions/runs/38040050699)
at `86d9f77840a935fb344009831ab6efbc8f73c35a` passed all eight source-native
tests without skips and the original Chromium contracts. Both initial captures
and zero-diff repeats retained distinct physical JSON/HTML reports and PNG
owners. Removing Right, then running Left against Right's old-shape basename
report, returned the explicit ownership refusal. Complete snapshot/report SHA,
size and modification-time inventories remained unchanged. This preparation
proof grants no full-source, protected-merge or publication credit.

The retained screenshot also shows the existing generic Playwright-install
hint for an ownership error and a removed Right entry still visible in the
sidebar after the actual API returned only Left. This report law does not
establish gallery removal synchronization or revise the shared error display.
