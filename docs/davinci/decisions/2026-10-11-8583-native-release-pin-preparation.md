# Native checks during immutable release preparation

Issue: [#8583](https://github.com/ubugeeei-prod/vize/issues/8583).

The official pinned operator opens its immutable source PR before creating the
integration PR, installing the durable pin and completing the source body.
The native check triggered by the opened event must finish authenticating that
same preparation before selecting the source-owned recipe.

The original v0.445.0 source PR #8577 opened at 04:29:48Z on 2026-10-11.
Native run [38111760853, job 114388620112](https://github.com/ubugeeei-prod/vize/actions/runs/38111760853/job/114388620112)
failed fetching `refs/heads/release-pin/v0.445.0` at 04:30:26Z.
The authentic pin `556c0d25dafa2cf128c25680d6a75b14dd94e454` was created at
04:30:47Z; the completed source body followed at 04:30:50Z. The failure preceded
every native workload. Its 197,378-byte raw log remains in retired archive
`05b9d64701023b15e0faeeebad8abc0bbdd70487`, at `failure/114388620112.log`.
The source H `605429c660b328bb03bec5bbfef15961e59fcaf0` stays retired and unpublished.

The helper waits at most five minutes for an originally incomplete preparation.
Every observation authenticates the live PR's repository, source, cut, version,
author, state, draft status and integration marker, together with the advertised
immutable source branch. A completed receipt with no pin fails immediately.
Once a pin is advertised, it cannot change or disappear during preparation.
Changed, foreign, closed, missing or timed-out authority never selects a recipe.
The observed pin is fetched by its exact object ID; both advertised refs are
checked again. The existing complete pin parent/tree/message, generated metadata
and source workflow/environment checks then select the whole original recipe.
The receipt retains the initial event and each preparation observation.
The dedicated handshake module is loaded from the authenticated workflow SHA's
Git object, including when the original wrapper runs its two helpers from a
temporary directory. Checkout module bytes cannot substitute for that authority.
The split keeps both source modules within the existing 350-line growth limit.

This fixes selection timing without changing workflow admission or native tests.
The unique tooling companion covers delayed readiness, immediate complete
readiness, finite timeout and foreign/stale/missing/altered authority. Existing
source-recipe and finalized-event controls remain mandatory. Fresh Actions and
actual protected delivery are required; this source correction grants no retry,
release or installed-consumer credit to the retired cut.
