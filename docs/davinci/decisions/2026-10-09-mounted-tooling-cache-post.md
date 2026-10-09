# Mounted tooling cache post

Paired issue: [#6830 decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6062958951).
This is a bounded correction to the existing
[Rust cache backend](./2026-09-27-rust-cache-backends.md), preserving the
required complete main Check and its strict result report.

## Actual failure

Two automatic Check runs on signed main completed the release-script tests
with zero failures, then exceeded the original 30-minute job deadline while
the primary Rust target cache action created its archive:

| Main                                       | Actual job                                                                                      | Script tests                                       | Archive creation before cancellation              |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------- | -------------------------------------------------- | ------------------------------------------------- |
| `81aa0c449cdeb2daf2959d3c9a272ff880bf4765` | [113331372722](https://github.com/ubugeeei-prod/vize/actions/runs/37783239113/job/113331372722) | 6,762 total, 6,700 pass, 62 existing skips, 0 fail | 13:41:10.519 to 13:47:54.763 UTC; 404.244 seconds |
| `9d974d27b6823772d36a6855557cbbc27b3c1cd1` | [113356227073](https://github.com/ubugeeei-prod/vize/actions/runs/37790211385/job/113356227073) | 6,772 total, 6,710 pass, 62 existing skips, 0 fail | 14:35:46.598 to 14:41:58.970 UTC; 372.372 seconds |

Both jobs mount the stable `test-scripts` target and perform a lookup-only
Actions cache miss before validation. The pinned combined cache action still
registers a save post, running `tar --posix -cf cache.tzst` with `zstdmt` over
`target`. Neither raw log records archive completion or upload before the
deadline. The mounted filesystem reports 173.95 GiB used; this is not a
measured compressed archive size. Cache action and policy source bytes are
identical across the two commits. Busy-unmount warnings occur after the
cancellation and do not establish its preceding cause.

## Bounded decision

Only the primary `test-scripts` role with a successful actual target mount
uses the existing pinned restore-only action with lookup-only enabled. Its
lookup preserves the clone and registers no archive save post. GitHub's
string comparison is case-insensitive, including this role comparison.

The same trust policy validates event, repository, checkout SHA and paths
before mounts. Registry/Git seeding, secondary targets, other roles, stable
provider names, Actions keys and literal paths retain their existing behavior.
A failed tooling target mount or trusted GitHub-hosted checkout still uses
the original combined cache action and can save a fresh per-SHA seed.
Every untrusted checkout remains restore-only and cannot mount or save a
writable provider cache. Cache misses still build and validate current input.

Tradeoff: a mounted trusted tooling target no longer exports a fresh per-SHA
Actions target seed. Existing compatible seeds and the original unmounted or
GitHub-hosted seed paths remain available. No cache deletion, role alias,
blanket policy change, migration, new job, retry, command or deadline change
is part of this repair.

## Controls and remaining qualification

All 25 existing focused cache laws retain their assertions; generic seed
witnesses explicitly use an ordinary role. Three additional laws exercise
18 mounted packets over the three trusted events, lookup miss/partial/exact
states and both role casings, reject the original save conditions, preserve
27 fallback/ordinary-primary packets with independent secondary tooling
saves, and reject unsupported fixture atoms or unbalanced groups. The fixture
supports this small condition grammar without evaluating arbitrary code.
All 28 laws pass on actual Node 22.18.0 and 24.14.0, with no skipped,
failed or cancelled tests. The configured focused format/lint check passes
with zero warnings. Independent review confirms that reversing just the two
conditions and their comments restores the original Action byte for byte.

TODO: qualify exact-source Actions, the protected queue and signed delivery,
then require the original automatic main Check to finish all four full JS
needs and its strict report. Local backend controls do not prove live post
omission or a measured wall-time gain. The independent `9d` package-test
failure remains separate, and a cancelled need never counts as success.
