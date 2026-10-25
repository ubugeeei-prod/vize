This active batch preservation corpus keeps the original published 5.4 page that
produced seven false TS2339 diagnostics in the official content mapper (#10526),
plus an invalid number/string assignment as a false-clean control. The batch
checker already accepted the valid page before this mapper fix. This pack
therefore makes no claim that it reproduces a batch regression.

The shared real-provider hydrator installs the exact pinned package and runs its
actual scanner before overlaying these unchanged page/config/declaration inputs.
Provider authority and the independently pinned official mapper-host lock are
retained with the reproduction. The named batch runner compares every diagnostic.

`tests/tooling/router-experimental-mapper-oracle.test.ts` is the mapper execution
witness: complete actual host and reference process packets, valid/invalid/repair,
explicit opt-in controls, effective root precedence, same-process isolation, and
exact Unicode LF/CRLF source mappings. No router declaration is fabricated.
