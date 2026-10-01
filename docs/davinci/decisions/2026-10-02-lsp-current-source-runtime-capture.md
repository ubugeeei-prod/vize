# Bounded LSP capture at accepted source — #6883

This record covers captured source
`514b7b51bbf531df628b272108a6b946f3045b58`, tree
`8ab6c95116397b1856bde5e6fbbcf087a3445a4b`, based on accepted signed main
`f634969712c435a9e4c07d405e35dcc89f29f1e0`.
The [earlier historical capture](./2026-10-02-lsp-source-runtime-capture.md)
remains separate and unchanged. This record's later documentation commit and
future fresh-main replays do not inherit execution credit for their new heads.

## Actual source-built execution

[Check run 36946673870](https://github.com/ubugeeei-prod/vize/actions/runs/36946673870),
attempt 1, built the CI-profile CLI from the captured source. Its
[test-scripts job 110650274158](https://github.com/ubugeeei-prod/vize/actions/runs/36946673870/job/110650274158)
succeeded with 5,440 passes, zero failures/cancellations and 12 unrelated skips.
The seven-session/ten-response scenario passed at `2026-10-02T00:45:25Z`.

The actual build receipt, report and every retained executable version probe
agree on source `514b7b51bbf531df628b272108a6b946f3045b58`,
`target/ci/vize`, `vize 0.429.2`, and binary SHA256
`dca35dc8c2d0eb5e35e69634e1dbe0446966b1de7c460e18174d43e73bcee364`.
The same job passed the real-tree ordinary-module gate and canonical
`rust-script` assertion lint under the unchanged committed allowlist.

The whole Check concluded failure: 18 successful jobs, one failed
`security-audit` job and five skipped jobs. The failure is node-forge HIGH
advisory `GHSA-86w9-cpqp-85rv`, affecting versions through 1.4.0.
A successful LSP job does not waive this failure or establish protected acceptance.

## Original retained bytes

The job retained artifact `11202878306`,
`formatter-api-corpus-36946673870-1-test-scripts-full`, 21,765,706 archive bytes.
GitHub's archive digest, printed by both the uploader and downloader, is
`e9b80c2146eab8e53b79ea247a5848e40efd7660aa744aed7d97aaf9159c5472`.

The read-only [transport run 36949056787](https://github.com/ubugeeei-prod/vize/actions/runs/36949056787)
and [job 110657678267](https://github.com/ubugeeei-prod/vize/actions/runs/36949056787/job/110657678267)
succeeded using the existing temporary helper
`ad2fb5c4b547d6b6905dc9b513da95c602a6a4d8`.
Its guards bind the actual source run, attempt, artifact and captured source head;
it reads the original artifact without building or launching another product.
No bytes from the earlier historical capture were reused.
This record does not claim local inspection of literal ZIP bytes.

Ten ordered lossless chunks recovered 98 original files, totaling 396,941 file
bytes: the build receipt, full seven-case report, and four files for each of
24 catalog-associated passive sessions. The 567,677-byte JSON and 86,532-byte
gzip SHA256 values are respectively
`e2b55e4b431aa4b5dc15be4a15daab1b9397d1ce5f7efaa388e6de5049055476`
and `89c260b09426a439c1a6a9609010e3cd43248faaa55804e64031ac113087e62a`.
All original file lengths, hashes and canonical base64 encodings verify.
The existing 2 MiB gzip, 32 MiB JSON, 16 MiB file and 128-file bounds remain.

The retained `differential/lsp.json` SHA256 is
`6e13b2ecd97bcb52a015063bd05f1b7a82cc56bcdf0b32c0471b3fd0edf246db`.
The actual frozen manifest loader and complete-response validator confirmed
seven matches, zero failed cases, zero baseline drift and ten complete equal
JSON-RPC response envelopes. All seven processes closed with exit zero, no
signal and no transport error; requests, readiness/order/lifecycle, full framing,
omitted properties and complete stderr remain retained.

The independent strict audit confirmed 24 distinct passive process closures,
72 untruncated length/hash-verified streams, 48 complete client/server frame
streams and 29 current caller witnesses. Recomputed callback/hash associations
cover all ten catalog origins: eight response candidates and two safety controls.
The 26 associations refer to 24 executions and grant no duplicate credit.
The catalog's historical source pin remains separate from the current build
revision. Original input/state/dependency reconciliation remains pending.

## Admission and delivery

This bounded capture is independently and parent-reviewed evidence for the
captured source above. Source-backed response capture and full passive transport
do not admit historical expectations, close fixes or establish native behavior.
The seven frozen scenarios still cover three selected historical fixes; the
255-fix / 447-nonmerge-touch ledger remains unchanged. The later #7385 Tag
response/state fixture remains an explicit unfinished obligation.

The four logical delivery layers remain provider, Actions consumer, existing
session capture and source association. After the security repair and other
provider prefixes reach actual accepted main, replay only these owned layers,
preserve every other owner's paths, and obtain fresh exact-head Actions.
Verify actual GitHub native Stack order, protected queue candidates and terminal
main merges before claiming delivery. Protected acceptance, whole-fix closure
and native handled/equivalent remain zero; #6883 stays open.
