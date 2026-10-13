# Mounted documentation example cache

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision removes
the duplicate Actions-cache archive for the documentation playground's mounted
example target. The existing trusted sticky disk keeps its original post-job
handling. No build, test, source receipt, diagnostic, budget or publication gate
is removed.

The selection is the existing `docs-wasm` primary role with `docs-example` as
its secondary role, and only after the actual secondary mountpoint probe
returns true. GitHub's expression comparisons retain their existing case
semantics. Other role pairs keep their original policy.

Use `actions/cache/restore` in lookup-only mode for this mounted secondary
target, retaining its exact old seed key, literal path, restore prefix and
compression-version inputs. Register no second archive/save post for it.
Keep the original cache/save action when the secondary mount fails or the
trusted runner is GitHub-hosted. PRs, merge groups and other untrusted sources
retain their complete original restore-only behavior. Primary targets,
registry/git caches, event validation and trust boundaries stay unchanged.

The sole runtime caller is `build-docs.yml`'s `build-playground` job. It mounts
`.docs-example-target` before building the Musea example. That example's
unchanged build runs `pnpm check`, which runs the current source through
`cargo run --target-dir ../../.docs-example-target -p vize -- check`, followed
by the original JavaScript checks and Vite build. Cache hits never select away
this check or accept a previous source's output. The composite exports no
qualification outputs.

The pinned sticky action is annotated tag object
`d58829ad6f9be8822ec5efe74256c82cd62915ab`, resolving to commit
`13af8883542ca949a717e70fef89d15edbb29d88`. Its
[original post](https://github.com/useblacksmith/stickydisk/blob/13af8883542ca949a717e70fef89d15edbb29d88/src/post.ts)
flushes and unmounts the provider disk, then requests a commit only if setup
and prior-step failure checks are clear. Failure or ambiguity skips that
commit. This change leaves the whole original mount/post registration and
provider behavior intact; it does not guarantee a later persisted clone or hit.

The measured original jobs are trusted `push` runs on the same Blacksmith
Linux/X64 role, with identical cache-action/policy and example recipe bodies:

| Source and original job                                                                                     | Example archive bytes | Example save phase |
| ----------------------------------------------------------------------------------------------------------- | --------------------: | -----------------: |
| [0e0839f, job114603463302](https://github.com/ubugeeei-prod/vize/actions/runs/38184376599/job/114603463302) |        25,300,056,260 |          514.854 s |
| [67679ee, job114609255684](https://github.com/ubugeeei-prod/vize/actions/runs/38186304640/job/114609255684) |        25,300,183,856 |          435.541 s |

The later job missed the same example seed key after the first successful
save, while its WASM seed hit and skipped its own save. The later example
post successfully unmounted and committed its existing sticky disk separately.
The jobs compile different source trees, so their total times are not a
matched performance comparison. The change removes the archive phase measured
above; a net workflow or PR speed improvement has not yet been measured.

Complete retained raw logs bind the observations:

- First playground: 211,505 bytes, SHA-256
  `1ace27590d1443e8c04ce290a4a1468578d45c7149e4e7e945b5508f3d34283c`.
- Later playground: 209,033 bytes, SHA-256
  `6ba5edc700232df8dd168c1f95caee52c0b14eda5f4e9e87be408232fbaa924b`.

A subsequent authenticated provider CLI census contained no example or WASM
seed and one 8,171,348,011-byte NAPI seed. GitHub's cache REST inventory cannot
establish those provider entries, because Blacksmith transparently serves
`actions/cache` through its own backend. The provider's
[storage documentation](https://docs.blacksmith.sh/blacksmith-caching/dependencies-actions)
does not establish this account's actual capacity, entry eviction, version
visibility or the cause of the repeated miss. No cache deletion, account
setting or quota change is part of this decision.

The regression executes the actual composite conditions and current policy
in real Git fixtures with separate current-source primary and secondary
subprocess workloads. The original action registers the unwanted example
save; the corrected mounted action omits it while preserving the same lookup
identity and both outputs. Trusted events, misses/partial/exact hits, failed
secondary mounts, GitHub-hosted fallbacks, PR/merge-group restores, old compatible
seeds, unrelated roles, an independently failed primary, malformed source/path
metadata and a genuine secondary exit23 remain covered. A Linux-only law uses
the actual `mountpoint` command to distinguish a filesystem from an ordinary
directory. Existing cache regressions retain their default workload behavior.

Qualification requires fresh exact-source Actions and all applicable
protected queue workflows through actual merge. Record the next ordinary
official Docs cache post after integration and keep any failed provider/save
receipt separate. Full Docs/public delivery, PR cache availability, a net speed
ratio and the cause of the original miss remain unfinished.
