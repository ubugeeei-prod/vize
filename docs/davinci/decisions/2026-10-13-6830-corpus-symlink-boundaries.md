# Differential corpus symlink boundaries

Issue: [CI #6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Consumer: [deprecation fix #8804](https://github.com/ubugeeei-prod/vize/pull/8804).

The original canonical SSR/Pug job on source `c5efbf9d4a` failed before
compilation observations completed: Jellyfin's `packaging/deb/root` link
points to its project ancestor, and recursive directory enumeration
repeated that alias until the filesystem returned `ELOOP`.
The complete original failure is retained in
[job 115101468288](https://github.com/ubugeeei-prod/vize/actions/runs/38337488540/job/115101468288).
Its dependent DOM aggregate correctly refused the failed observer tier.

The collector now enumerates each physical directory edge at most once
along a logical route. Independent in-root directory and file aliases keep
their logical paths and whole source bytes. A resolved directory or file
must stay inside the canonical selected root; links to outside sources do
not extend the pinned corpus. Physical directory entry names are joined to
the logical route, so resolving an ancestor link does not send an expanding
alias chain through one filesystem lookup.

This changes finite discovery, not the compiler or its oracle. The original
source reader, inventory reconciliation, scope classification, exclusions,
commands, selectors, and budgets stay unchanged. Original IO failures still
retain their complete logical path and OS diagnostic. Three additive Unix
controls cover the Jellyfin ancestor link, independent source aliases, and
outside directory/file links, including whole Unicode and CRLF source bytes.

Root source review accepted the complete production/test bodies in proposal
`3a3db5a70a85d25f6517cf94200cbcccd7fb0b8c9eeff831b8bff5b813172cdd`.
Only Rust formatting/parser checks have run locally; the new Rust controls
and unchanged real corpus must qualify on ordinary Actions and the protected
merge queue. A source review does not establish native acceptance or a
performance improvement. The original failed source will not be rerun.

After actual merge, #8804 will compose this genuine prerequisite once and
run fresh checks on its own source. Its retained original `ELOOP` and
propagated aggregate failures remain separate from successor qualification.

The first source Check on `3efdca55a4` refused the production helper's
`expect` under the existing `clippy::expect_used` gate. The complete
140,521-byte original log is retained with SHA256
`672d0c2abf21df25934d642d222ec179b6182cf22d6bf91cb5b1cea5c1235e38`.
The successor handles an absent entry name with an explicit refusal; traversal,
all three controls, and the original lint policy remain unchanged. Fresh
Actions must qualify that successor; the first failure grants no test credit.

The `fb67f45784` source Check then exposed an observer integration failure:
the standalone harness omitted `collect_vue_edges`, and removing an IO
helper was no longer rejected because its caller was also omitted. Extract
the exact transitive helper bodies and refuse missing or duplicate helpers.

Finite discovery intentionally changes the repeated ancestor-loop path
vector. It must not silently replace the original oracle. Keep the complete
original kernel-limited physical vector, its whole bytes, and its independent
committed Git walk. Archive its raw NUL vector and file rows separately,
with both hashes bound into the native receipt and source identity. The
original hosted census remains 44,367 routes, including 5,576 Jellyfin
routes; these observations do not become successor performance evidence.

Qualify the successor against a separate committed edge walk and physical
whole-byte traversal. Its entire ordered vector must equal the original
vector after removing only subtrees beneath authenticated repeated physical
directory edges. Every original resolved source and Git blob must remain,
and each original source's SHA256, byte length, and blob must match a retained
successor. Missing files, reordered routes, uncommitted additions, replaced
archives, changed source/helpers, foreign links, and arbitrary cycles remain
refusals. The frozen-original Rust vector and literal IO controls remain,
with additive finite-successor controls. Source IO qualification now uses
the finite native vector while retaining the original census separately.

The original Rust source reader already resolves path components before
its one full UTF-8 read; the observer integration leaves that reader and
the production collector unchanged. Focused local harness checks use the
pinned Rust 1.99.0 directly, without a Cargo build. Full canonical inputs,
ordinary Actions, and the protected merge queue still require fresh
qualification on the repaired source.
