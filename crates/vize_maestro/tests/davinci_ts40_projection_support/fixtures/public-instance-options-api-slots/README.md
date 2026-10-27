# Public-instance slot-read transition

These are complete, unformatted Maestro packets and rendered projection records
captured before advancing the `options-api-slots` TS-40 snapshot for #10933.
Generated `.ts` and authored `.vue` packets have a `.txt` suffix for storage only.

The before source is `19828f974f80e857f74566078947fa5981c6b26b`, captured by
[run 40484190377](https://github.com/ubugeeei-prod/vize/actions/runs/40484190377),
[artifact 12724177839](https://github.com/ubugeeei-prod/vize/actions/runs/40484190377/artifacts/12724177839).
Its Canon, Croquis, Maestro, TS-40 fixture trees and Cargo.lock are identical to
the PR base `481c94bd55a7276339adc277edd93e30313f6ea9`; their Git object IDs are
retained in `manifest.json`.

The after source is the CI merge `1343403f3f7628f0d9c8d20b0ac1513fc1277d19`
of PR head `fb6a52ed0e3f6dd737e375c57cb991232d6f565a`, captured by
[run 40486957980](https://github.com/ubugeeei-prod/vize/actions/runs/40486957980),
[artifact 12725382688](https://github.com/ubugeeei-prod/vize/actions/runs/40486957980/artifacts/12725382688).
Both artifact SHA-256 identities and a byte-length/SHA-256 comparison of all 194
packet files across the 12 fixtures are retained in `manifest.json`.

Exactly four packet files differ: the Maestro template, document summary,
mappings and authored hits for `options-api-slots`. Canon, the content mapper,
the Maestro script and all other fixtures are unchanged. The template now
declares `message` through `__VizeTemplateInstance` and reads the public-instance
member once at each authored occurrence, `147:154` and `167:174`. The predecessor
retained only the first explicit unresolved read. The second mapped read adds
one mapping (7 to 8) and one authored hit (6 to 7); all prior authored coverage,
feature flags and diagnostic witnesses remain. The extra declarations shift the
first generated read from `17139:17146` to `17553:17560`, and the additional read
lands at `17633:17640`.

The frozen before record remains alongside the complete before template/maps;
the current snapshot only advances the three reviewed Maestro record fields.
The packet authentication test checks the retained files and binds the complete
after record to the live generator. Both existing mapping and diagnostic drift
controls remain in place.
