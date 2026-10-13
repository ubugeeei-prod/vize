# Bound stalled rust-script asset transfers without changing native authority

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Check [38347489618](https://github.com/ubugeeei-prod/vize/actions/runs/38347489618),
attempt 1, stopped tooling shard 4 before any tests started. Its exact head was
`8fa23ba197df9df384f9375b4fab69852d788505`; the setup download ran for
622,176 ms before curl refused with status 56, HTTP 200 and a receive timeout.
The [complete raw job log and control custody](./fixtures/rust-script-transfer/custody.json)
remain retained. No body checksum or loaded native runner is inferred from this
failed download. A targeted rerun of that unchanged head succeeded separately.

Current main `e78d2cb93e278032cd262201b0a444273618ee25` has the same
unbounded downloader. Upstream asset [127872817](https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817)
is the pinned 1,255,442-byte rust-script 0.34.0 Linux x86-64 archive.

## Decision

Apply a 10-second connection limit, a 30-second per-transfer limit, and a
five-second low-speed period below one byte/second. These are curl's actual
[connection, transfer and low-speed controls](https://curl.se/docs/manpage.html).
Retry the same primary URL once after one second only for transient network
failures with their exact expected HTTP field: connect/empty reply with 000,
partial body with 200, or timeout/send/receive failure with 000 or 200.
TLS, authentication, missing-asset, rate-limit and malformed-status refusals stay
unchanged. The existing curl 22 / HTTP 500, 502, 503 or 504 recovery still makes
one request to the same pinned GitHub asset API; it gains the same deadlines.
There is no all-errors retry or additional mirror route.

Each attempt clears a private staging file in the archive's existing directory.
Only a complete successful transfer renames it onto the caller's archive. A
contained subshell trap removes partial output while preserving the install
action's original temporary-directory trap. Two primary transfers allow 61 seconds of curl limits and retry delay. If the
second primary returns a reviewed 5xx, the preserved single API recovery adds
30 seconds: at most 91 seconds of curl limits and retry delay.
This is a transfer bound, not a promised job walltime under arbitrary scheduling.

Version 0.34.0, Linux/X64 selection, primary URL, asset ID, HTTPS/TLS settings,
expected archive SHA256, complete membership/type checks, installation mode and
post-install version validation remain unchanged. Successful HTTP transport
still grants no native authority until the existing extractor authenticates the
entire body. Existing cache and other-platform installation policies are unchanged.

## Actual controls

`tests/tooling/setup-rust-script-transfer.test.mjs` runs the actual shell helper
and system curl against a trusted local TLS server. A test-only destination/CA
adapter preserves all production URLs and real deadline flags; it does not
shorten deadlines, fake curl statuses, disable TLS validation, or synthesize
curl output. Complete native curl receipts and public certificate/fixture bytes
are retained separately from the original Actions packet.

The original downloader is frozen byte-exact. Its genuine partial HTTP 200
response fails without retry. The candidate recovers the same endpoint, and a
real stalled HTTP 200 body hits the actual low-speed refusal then succeeds on
the second request. Two stalled responses retain the original owned archive,
return failure, and leave no staging file. Successful recovery followed by the
unchanged production digest refuses the benign fixture before extraction. A
separate explicitly labelled benign-fixture digest control exercises real tar
membership and extraction without claiming a loaded Linux provider.

Existing complete installer controls preserve all four reviewed 5xx routes,
authentication/rate/TLS/malformed-status refusal, API failure, whole digest and
membership/type boundaries, installed mode, version, and no Cargo fallback.
All suites run without Cargo/npm installation. Fresh Actions, protected queue
and actual merge remain pending. This does not grant release or native consumer
qualification.
