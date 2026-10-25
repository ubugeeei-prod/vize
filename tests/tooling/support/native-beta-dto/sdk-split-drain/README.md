# Local SDK split-drain experiment

This is an isolated validation copy of `corsa_client 2.0.0-beta.1`, registry
checksum `3e4a1ace5347ca6717e952f6b2fbc73ef46e641a8f3ce1143d13c366880381e1`,
upstream VCS `1c5d10bd71b2d5ca2a0d885abb14fcf8d4381d1f`.
It is **not a production dependency, a released SDK, or an upstream proposal**.
`stock-custody.json` pins every original registry file. `sdk.patch` changes only
`src/api/client.rs` and `src/api/snapshot.rs`; the new unit-test module is separate.
The global Cargo registry remains unchanged. The validation branch's local
Cargo override and the exact two-line lock delta must never be copied to a
production/release branch as an incidental dependency update.

The stock source exposes `spawn` and `connect_pipe`, but no public transport
injection hook for replacing its full-duplex Unix reader/writer ownership.
`ManagedSnapshot::release` relinquishes a wrapper's lease; the DerivedSnapshots
lineage can still retain that same head. Stock `ApiClient::close` clears the head,
drains its release queue, then closes/joins its transport. An attached external
LSP must stay alive for that drain, while its socket peer must close before the
SDK reader can be joined. The split makes those two existing phases callable
separately, with the existing `close` reusing both phases.

The stock source-workspace observation is run `40235959056`, source `9eab4a27`,
artifact `12614679184` (ZIP SHA256
`d4027aed8efd68c0d6bf3adefc5ba9870ba6acff2a02d8a73baa58ab00b25622`).
Its selected-config native owner 6826 creates snapshot 2, derives snapshot 3,
releases only 2, then ACKs shutdown and observes stdin EOF at
`1792964153.572854`, exiting 0 at `1792964153.573501`.
The source emits the snapshot-3 closed-JSONRPC warning at
`1792964153.583651`. Cached ProjectSession ownership of head 3 is source-grounded
inference, not an observation of a private Arc count.

The terminal caller must quiesce every API user and explicitly release all
retained snapshot wrappers, including cached sessions and cloned leases, before
`prepare_close`. That call clears the existing lineage and waits for the existing
release worker without joining the transport reader. Native shutdown/reap and
socket EOF then precede the existing `close` reader join. The positive proof must
show actual release ACKs for **every created head**, before native shutdown.
Worker completion alone is not proof of remote release: stock per-release errors
still emit warnings, and the source campaign must reject them.

This is not a reusable reset. Requests and late explicit releases after the
terminal stage are refused using the already-closed queue state. A late dropped
lease retains the stock warning; it does not become ownership success. Concurrent
in-flight requests during the terminal stage violate the quiescent caller
precondition; this patch adds no global lease tracker or concurrent API shutdown
contract. Fast local Unix IPC tests exercise real cached ProjectSession/snapshot
clones and terminal refusal, but carry no native-SDK functional credit.

The validation-only Vize cleanup changes drain configured API snapshots before
the normal native-owner close and retain the reader join after reap. Immediate
or emergency discard paths are unchanged. Maestro's final worker/owner barrier
(#10760) is separate. The original materialized dependency-close/import failure
2307 remains an explicit strict failure; this experiment does not readmit that
closed dependency or claim a complete beta editor qualification.

The later validation composition adds only the exact queued normal-retirement
private method and bridge callsite from `ed44709453b1561a0420507b6d79284f3e507357`.
`../queued-retirement` retains its 702-only source authority and the actual two
hunk composition hashes. No whole client file, initial-diagnostics change or
10760 final outer barrier is inherited. Existing immediate/emergency discard
callers retain their behavior. This dependency is not itself beta acceptance.
