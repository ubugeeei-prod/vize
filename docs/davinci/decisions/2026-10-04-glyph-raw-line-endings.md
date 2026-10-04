# Preserve raw template line terminators (#7745)

Issue: [#7745](https://github.com/ubugeeei-prod/vize/issues/7745).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).
Node bridge: [#7736](https://github.com/ubugeeei-prod/vize/issues/7736).

Original [#963](https://github.com/ubugeeei-prod/vize/issues/963) requires
`pre`, `textarea` and `v-pre` content byte-for-byte, with only surrounding tags
changed. Existing template emission copies that content verbatim and the SFC
block writer repeats the same contract. `endOfLine` selects layout; it does not
revoke raw fidelity. HTML can normalize CRLF to LF, so the observed pre example
proves a source-fidelity violation rather than changed rendered text.

Actions Check 37176492863 at Node bridge head `9670d2a1` proved a core mismatch
through both actual exported Node and CLI: independent reference 170 bytes,
actual 171, inserting CR at byte 71 inside an authored LF raw body. Actual output
SHA-256 is `281c73392491e1c14060cc15d259df25522796afa6548bdea348b7ead5cda744`.
Retain that original independent reference, not a recaptured printer result.
Use explicit CRLF in the lower core fixture to reproduce without the Node bridge.

Rejoin raw continuations using authored separator bytes. The existing raw mask
owns the state at a line's start: a separator entering a raw line is raw, while
layout resumes after its closing line. The cold raw helper splits LF, CRLF and
lone CR without discarding separators; formatter-owned boundaries use configured
layout. The ordinary LF emission path remains specialized. Public Rust laws
check complete output, blank raw lines, all three styles, all Vue versions,
unchanged options and three-pass fixed points. Original four owner test bodies
and historical pins remain unchanged; register only the exact new complete owner.

Add one legacy CLI corpus case; preserve the original eight cases, 300 API plans,
all historical captures and zero native acceptance credit. Node #7740 branches
from this lower repair in a native Stack and proves public option forwarding
against the shared independently authored whole output. Both exact heads and
all 104 unchanged instruction ceilings must pass before entering a ready prefix.
Release freeze holds all new queue admission until publication/thaw, and actual
protected queue merge is required before closing the scoped issues.

TODO: broader profiles, source maps and arbitrary option combinations remain
under #6098; this repair changes no source-map/result API or public option scope.
