# Path host ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

Owning [decision](https://github.com/ubugeeei-prod/vize/issues/6834#issuecomment-6050401219).

Continue the independently ready Stage 1 host-boundary work after the
filesystem adapter delivery in #8230. The path module canonicalizes real OS
paths and retains Windows drive/UNC spellings for CLI and native type-checking
callers. No normal level consumer requires it.

Move the complete actual-main `path.rs` from L0 to Carton in a move-only
commit, then run the bounded path replay for integration. Preserve every
function body, target cfg, fallback, input and all four original Windows
prefix laws. The existing private `cfg(any(windows, test))` string helper
inherits a function-scoped type-lint expectation at its actual host owner.

All 56 direct Rust callers remain in their existing host crates. Script their
103 qualified references and four grouped imports to the same three APIs;
preserve every caller operation and existing dependency declaration. Keep the
oversized cstr imports on one line through a cross-crate grouped import.

Admit only those three APIs at the complete physical caller/count inventory.
Unknown paths/APIs, extra calls, module aliases, wildcards, storage and literal
references remain rejected. Compose the real Maestro path caller with its
existing source-IO exception; preserve the profile and i18n admission clauses.
Remove only the three obsolete L0 std-string bridge rows and regenerate the
existing consumer inventory from actual sources.

The historical allocator replay authenticates the complete new Carton facade,
removes only its added path/source-IO declarations in the replay view, and
requires the unchanged original allocator facade again. Its original replay
fixtures, accounting implementation, host selection and negative controls
remain exact. Current source checks never replace the current facade with
historical bytes.

Require fresh exact-source Actions to execute all four named Carton path laws,
the original formatter Git-metadata test and both Maestro discovery/inventory
laws. The native qualifier, complete canonical corpus, unchanged allocator
controls and all 104 instruction cases still qualify the protected candidate
before actual signed delivery. Local source checks grant no runtime credit.

Whole #6834 stays OPEN: native Timer/global profiler, TLS/locks, arena pooling,
recursion and remaining configuration/platform isolation need their own
provider and ownership work. This slice changes no product route, stage,
parser, fixture, manifest or numeric ceiling and grants no whole `no_std`,
native/default graduation, release or installed-consumer credit.

The first source head `d69880764bc7a655a9da86a797d187c15aa8a4f5` executed
all seven original path/Git-metadata laws once across 16,874 passing Rust tests.
Its old Canon alias assertion still counted the moved OS call as L0; retain
its original total of two sites as one L0 storage site plus one exact Carton
path site. Its distinct-host preflight refused the new production footprint
before any measurement. Keep both failures as historical evidence and require
a fresh complete successor; neither observation grants delivery credit.
