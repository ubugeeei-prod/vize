# Avoid archiving healthy mounted Rust targets

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision extends
the existing mounted `test-scripts` and docs target behavior to every Rust
target role, including an independently mounted secondary target.

## Measured problem

On main `cd036618173f1b3598e3b656e6f2a21854efd4a2`, `check-vize-apps`
[job 115351091358](https://github.com/ubugeeei-prod/vize/actions/runs/38412852483/job/115351091358)
had a healthy mounted target and a lookup-only target seed miss. The registered
Actions cache post then compressed 14,571,230,028 bytes: tar started at
14:27:18 UTC on October 13 and upload started at 14:34:45. Upload and final
sticky cleanup finished around 14:35:12, about 474 seconds after tar started.
The CLI build took about 24 seconds and its workload about a minute. The large
archive was extra work after successful validation.

## Decision

When the existing mount check reports a primary or secondary target mounted,
use `actions/cache/restore` with lookup-only mode. Keep the compatible stable
seed key and restore prefix, but register no target archive post. The original
sticky provider still persists trusted successful builds and discards candidate
clones. A seed miss does not change that behavior.

Evaluate each target independently. A failed mount or a GitHub-hosted runner
retains the original exact-source key, restore and trusted save. Existing seeds
remain compatible fallback inputs. PR and merge-group trust rules, cache paths,
namespaces and versions do not change. Registry and git caches keep their
existing behavior. No cache deletion or build/check removal is involved.

## Validation

The existing cache fixture checks both mounted targets across source changes,
old stable and per-source seeds, failed primary and secondary mounts, hosted
fallbacks and read-only candidates. Nested-post checks continue to verify exact
input paths using nonmounted targets. The cold negative reader discards its
prior-run fixture marker along with the removed target files, preserving the
same-key/different-version refusal.

The historical archive duration establishes the bottleneck. A fresh trusted
main run after merge must confirm the removed archive post and measure its
remaining sticky cleanup time; this change does not claim that future result
in advance.
