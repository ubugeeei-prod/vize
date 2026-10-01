# LSP source-built runtime and retained-byte proof — #6883

This dated proof covers the exact historical source
`3d5845521dc89e5e7cb95aec7d2566dfe4c0e671`, tree
`2a981294dc5829e1d4546da26ddd953ac13585f1`, anchored on signed actual main
`130ef0eec3873629c6675cfa4c7190489551ad55`.
It preserves the scope of the [shared response adapter](./2026-10-01-lsp-shared-response-adapter.md)
and [existing-session associations](./2026-10-01-lsp-retained-session-source-links.md).

## Actual runtime

[Check run 36931497712](https://github.com/ubugeeei-prod/vize/actions/runs/36931497712)
source-built the CI-profile CLI and executed the actual
[test-scripts job 110601553493](https://github.com/ubugeeei-prod/vize/actions/runs/36931497712/job/110601553493).
The source-bound seven-session/ten-response scenario passed at
`2026-10-01T22:02:42Z`; the complete tooling job passed at `22:08:44Z`, with
5,435 passes, zero failures/cancellations and 12 recorded unrelated skips.

The actual printed source-build receipt identifies `vize 0.429.2`,
`target/ci/vize`, SHA256
`9d77cf55b2e5bf004aa6aff05ebbd24cb8eb5602ed0968239a05fda4b1bd0bbd`.
The retained receipt and executable version probes independently agree with the
same source, executable identity, version and successful process statuses.

## Original artifact and byte audit

The job retained original artifact `11196788005`, named
`formatter-api-corpus-36931497712-1-test-scripts-full`, 21,024,535 archive bytes.
GitHub's archive digest is
`49690b5fd346b16e78360aaeacee32fdd785c6f971f303d1aaf8699462e0d839`.

The read-only [transport run 36936195727](https://github.com/ubugeeei-prod/vize/actions/runs/36936195727)
and [job 110616922765](https://github.com/ubugeeei-prod/vize/actions/runs/36936195727/job/110616922765)
passed at temporary transport commit
`ad2fb5c4b547d6b6905dc9b513da95c602a6a4d8`, a direct child of the source above.
The reviewed temporary branch changes only its private workflow and transport
helper. It downloads the original artifact and emits original retained bytes;
it performs no server build, replay, response generation or corpus promotion.
The pinned downloader also prints the matching archive SHA256 on the runner.
This record does not claim local inspection of literal ZIP bytes.

Strict framed transport recovered 98 original files: the build receipt,
complete seven-case report, and four files for each of 24 catalog-associated
passive sessions. Ten ordered chunks carry 567,677 JSON bytes and 86,580 gzip
bytes. Their independently verified SHA256 values are respectively
`4e5d5787339e0d2f84f9d020c227b644d8a65c76d9d9bd6ce3b6db07a7e0a2a7`
and `ccd755b289ae5448d9806beb8d4871bbd8c1eec6a5cc15998cde174bd68e0d07`.
The fixed 2 MiB gzip, 32 MiB JSON, 16 MiB file and 128-file bounds remain intact.
Every original file length/hash and canonical base64 encoding verifies; missing,
reordered or altered chunks and unsafe/duplicate paths are rejected.

The original `differential/lsp.json` SHA256 is
`77211e7234bd994d250daf0549c1e98b1222b4369af265dd6760153ec9c3b6b1`.
The actual frozen manifest loader and complete-response validator re-audited
these retained bytes against every immutable fixture input and expected result:
seven matches, zero failed cases, zero baseline drift, ten equal complete
JSON-RPC response envelopes. All seven client/server streams decode completely,
retain their hashes, authored requests, readiness/order/lifecycle and omitted
properties, and close with exit zero, no signal and no transport error. Complete
stderr bytes remain retained alongside both directions of framed transport.

All 24 associated passive observations also retain full untruncated streams,
complete client/server framing and successful process closure. Actual source
file hashes, selected callback witnesses, version probes, source/build receipts
and reconstructed source associations verify. They cover all ten catalog
candidates: eight response witnesses and two explicitly labelled controls.
Their original-input/dependency/state reconciliation remains pending; successful
transport and association do not admit historical expectations or close fixes.

## Delivery and remaining scope

The original full Check failed solely on the inherited external Maestro Tag
fixture's `.into()` useless conversion at `global_tag_names.rs:248`. The actual
T1/retained-byte result is evidence for its historical head. Delivery requires a
fresh current-main replay, exact-head Actions, verified native Stack membership,
protected queue validation and actual main merges.

The seven registered scenarios have three selected historical fix witnesses:
inactive imports, CRLF formatting and folding. All 255 selected fixes / 447
nonmerge Maestro touches remain retained, including all response, control and
internal obligations. Whole-fix closure and native handled/equivalent remain
zero. #6883 and the native whole-product LSP remain unfinished. Complete
initialization, diagnostic/state/version traces, close/reopen, cancellation,
UTF-16/racing-state guards, checker non-start, shared dependency/helper closure,
Canon/editor/provider histories and the later #7385 Tag response/state fixture
remain separate admission obligations.
