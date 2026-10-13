# Retire replays of completed host moves

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The timing observer, path and profile-export host moves and the profiler
allocator selection are complete on `main`. Their tooling tests still replayed
each move against the live tree and compared whole-file digests of the moved
sources and their callers, so any edit to `walk.rs`, the Curator profile
caller, the carton allocator or the CSS naming gate failed them without a
behaviour change.

- `profiler-allocator-ownership` is deleted; every test in it was a replay.
- `timing-observer-host-boundary`, `path-host-boundary` and
  `profile-export-host-boundary` keep their host import boundary tests: no
  Davinci source may import the host runtime outside the admitted declarations,
  and aliases, wildcards and extra APIs stay rejected.
- The move scripts under `tools/support/levels/` and
  `tools/support/compat/levels/` stay for rebasing older branches; they are no
  longer asserted to be a no-op on the live tree.
