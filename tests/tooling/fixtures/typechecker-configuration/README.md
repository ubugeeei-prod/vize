# Typechecker qualification evidence

`n8n-original.json.gz` preserves the complete `n8n-typechecker.json` emitted by
[Real Project Matrix run 41640196425](https://github.com/ubugeeei-prod/vize/actions/runs/41640196425).
`receipt.json` identifies the exact source, artifact, uncompressed SHA-256 and complete diagnostics.
The fixture contains tool output and paths, without upstream source contents.

The original coverage is structurally valid: 1,369 requested SFCs and 3,362 enumerated files.
Its four TS2688 errors and one TS6053 error report unavailable project inputs, so enumeration
does not qualify semantic checking. The subprocess controls retain this complete diagnostic
packet and its coverage. They also retain authored errors in mixed packets, reject those packets
from semantic gates, and qualify ordinary authored errors after configuration preparation.

Unavailable modules and generated declarations prevent qualification because unresolved types
can introduce `any` into later checks. This classification does not decide whether a missing
module originated in an authored import or fixture preparation. The original error remains in
the raw report for diagnosis; coverage validity and semantic qualification remain separate.
