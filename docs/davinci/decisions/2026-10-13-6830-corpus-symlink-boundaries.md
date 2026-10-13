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
