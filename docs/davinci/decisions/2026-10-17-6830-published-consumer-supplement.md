# Archived one-time consumer proof proposal for published v0.457.0

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Status: superseded before hosted execution by [#9311](https://github.com/ubugeeei-prod/vize/pull/9311)
and [#9319](https://github.com/ubugeeei-prod/vize/pull/9319). The duplicate workflow,
script, historical-reader entry point and their dedicated tests were removed.
Use the [canonical consumer recovery procedure](../../release/consumer-fixture-recovery.md).
This record and its retained failed-run evidence do not authorize source closure.

The original proposal remains in commit `4c16d62f4ad4cc1569ce71a3af48c9887e2da290`
and its integrated copy `e9cea98b2a7f1da1add55ddfcc7ff094b2827037`.
At supersession, R's latest attempt was 4, also failed. The attempt-3-only proposal
would refuse that newer attempt; it was never dispatched or accepted as recovery.

## Historical proposal

The original [Release R38805261385](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385)
at attempt 3 has 54 successful jobs, one failed Deno consumer, and one deliberately
skipped manual crates handoff. The failed job is
[116687943513](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385/job/116687943513).
All 42 original artifacts and the original failure remain evidence. Source
[#9196](https://github.com/ubugeeei-prod/vize/pull/9196), its H, pin and annotated
tag must remain unchanged. The P0 fixes merged later require a separate release.

The frozen Deno configuration encounters both the fresh-package age policy and
manual node_modules resolution. [#9257](https://github.com/ubugeeei-prod/vize/pull/9257)
fixed the current consumer configuration while preserving the original assertion
program. Its [successful public check](https://github.com/ubugeeei-prod/vize/actions/runs/38904123567)
supports the repair, but does not itself bind all historical release identities.
This proposal would have replaced only the remaining-publication-gate advice in
[the source restoration record](../../release/2026-10-17-source-receipt-recovery.md):
waiting and rerunning old workflow bytes does not apply the configuration repair.
The source restoration and immutable-source guard remain unchanged.

The proposed main-only workflow would run the exact H consumer against public
`jsr:@vizejs/vize@0.457.0` using pinned Deno 2.9.7, an empty cache and an empty home
outside the checkout, without workflow credentials or provider overrides. Its
configuration retained the external one-day dependency age policy and exempted
only the exact owned closure. It compared all seven JSR facade files with H and
authenticated H/C/pin/tag/R, signed integration V and H/V catalog equality before
and after the consumer. Removing this unused reader restores the existing main
implementation; ordinary promotion retains its strict catalog guards.

The complete original API/log capture is retained in
[published-0.457.0.json.gz](../../../tools/support/release/incidents/published-0.457.0.json.gz).
Its decompressed SHA-256 is
`9db9805310dff2f7d5bf3363274b7297df49206e0c483ccf7a170e03f1eab8a5`.
It records the original run, all jobs/steps, all artifacts and the raw Deno log,
captured read-only on October 16 UTC. The proposed refusal controls rejected newer
attempts, additional failures, missing or changed evidence, and unauthorized
runtime/config inputs. The capture remains historical evidence; it does not
represent the latest original attempt or a successful consumer proof.

## Current finalization boundary

Coordinate with the current release owner and follow the canonical procedure's
fresh signed recovery and installed public acceptance requirements. Before any
manual closure, independently recheck source #9196, H, pin, annotated tag, latest
original R/attempt and every public publication channel. Keep R failed and retain
all historical job/log/artifact identities, including the attempt-3 capture here.
Neither this archive, a proof artifact alone, nor code removal establishes release
completion or the original ten-minute end-to-end release target.
