# Patch devalue before release

Issue: [#7383](https://github.com/ubugeeei-prod/vize/issues/7383)

The required npm production audit on #7382 reported three high and two moderate
devalue advisories newly added to the GitHub advisory database on 2026-10-01.
The existing workspace override pinned Nuxt and Nitro's serializer to 5.9.2;
all five advisories identify 5.9.3 as patched.

Update that existing override and its lockfile resolution to 5.9.3. Keep the
audit threshold and advisory policy unchanged. This is a separate dependency
commit in the current serial PR because the audit blocks that PR and release;
it does not alter the formatter configuration or sorting behavior.

The lockfile update passed the existing supply-chain policy. Exact-head Actions
must pass the production audit and frozen-lockfile package/Nuxt checks before
merge. The alpha release still requires all its original publication gates.

Primary source: [shared-memory serialization advisory](https://github.com/advisories/GHSA-j22f-vq7h-c4qm).
