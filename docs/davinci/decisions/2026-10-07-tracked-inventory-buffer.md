# Complete differential-feature tracked inventory

Decision owner: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The protected merge group first failed on `d19c65e0fff3f872038e680de0cd7e60d85889ff`
in [tooling job 112786812234](https://github.com/ubugeeei-prod/vize/actions/runs/37619740336/job/112786812234).
Its differential-feature rename check stopped at `spawnSync git ENOBUFS` before
validating any selector. The cumulative candidates for #8149, #8160 and #8167
retained the same failure. Their individual source checks were green, but
these protected failures grant no delivery credit. The four PRs were removed
from the queue; the preceding #8135, #8141 and #8152 genuinely merged.

The exact primary candidate's complete Git tree contains 14,624 selected paths
and 1,049,989 bytes of UTF-8 NUL-separated names: 1,413 bytes beyond the pinned
Node 24.14.0 synchronous stdout default. Prior signed main `a4fa9a45` had
14,180 selected paths and 1,005,003 bytes. This is deterministic inventory
growth in the cumulative candidate, rather than a reported selector mismatch.
The primary raw log SHA256 is
`8714494d3fbd97abe61daa7ca1f1c92b5be35b3d89ba48b58e554901e4cd0e42`;
the #8160 and #8167 raw witnesses remain
`c930618655e221cd5c043f6b7182a4aec4845dd494cfad91072bb3e2d09a0b27`
and `f650f1937ec7e86015090b246dcbebb1db1fe6d4f69ee7a6a5e78644a36630f1`.

Write Git stdout directly to an owned temporary file, close its descriptor,
read the complete inventory, and remove the directory in `finally`. Preserve
the original path selectors, eligible-file policy, protected-block validation,
rewrite rules and check/write status. Git failure still rejects the operation
before any source write. Increasing a fixed buffer or dropping paths would
leave the same completeness problem for a later corpus.

A real temporary Git index supplies more than one MiB of tracked names and
places an eligible old selector at the final sorted entry. The unchanged
default buffered Git read must reproduce `ENOBUFS`; the repaired checker must
find that last entry, reject check mode without changing it, rewrite its full
contents, and then pass check mode. Existing published-edge and all-manifest
prevalidation controls remain, with a separate failed-Git no-write control.
Local focused results do not qualify a fresh source or protected candidate;
those exact-head Actions and actual signed merge remain required.
