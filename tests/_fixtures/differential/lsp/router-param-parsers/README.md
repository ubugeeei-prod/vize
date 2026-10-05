# File-route parser regression corpus

The original `Reported.vue.txt` is the complete SFC reported in issue #7816 by
[naitokosuke](https://github.com/naitokosuke) for
`src/pages/users/[id=int].vue`; its bytes are pinned in `references.json`.

The seven complete sources, diagnostics and completion responses are authored
from the public Vue Router param-parser naming contract and the existing Vize
modifier/detail contract. They were not captured from the implementation under
repair. The real stdio test opens each source, checks the complete versioned
diagnostic publication and completion result, changes it to an unknown param,
and restores it. The original source also exercises the existing CLI lint path.

This new legacy-product regression corpus is separate from the unchanged
historical shared-response manifest: that manifest currently supports request
responses, while diagnostics are server notifications. These cases grant no
native admission or whole LSP fix-history credit; #6883 remains open.
