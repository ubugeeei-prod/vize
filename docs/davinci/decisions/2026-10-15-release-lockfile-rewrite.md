# Release lockfiles use the textual version bump

The pinned version integration is accepted only when `Cargo.lock` matches
`rewrite`: vize package `version` lines change from the old release to the
new one, and every other byte stays. `cargo update --workspace` also rewrites
unrelated packages, so that integration was rejected and no tag was created.

The release preparation now applies that same textual rewrite to the root
lockfile. It does not resolve the registry again.
