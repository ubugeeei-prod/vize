# One agent runtime for repository work

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

On 2026-10-13 a second agent runtime (a codex session) worked on the CI
workflows in parallel with the Claude lanes. Both pushed to the same pull
requests, trialled merge-queue settings and closed each other's PRs, which
caused duplicated fixes and rebases.

The maintainer decided on 2026-10-14 to run repository work from one agent
runtime only. The codex session was stopped. Its open pull requests were
merged where they were complete (#8887, #8900, #8904, #8927) and otherwise
closed with a reason (#8903, #8922, #8966).

On the same day the maintainer set the merge queue to build at most 4 and
merge at most 5 entries per group, to reduce the speculative rebuilds that
queue jumps caused.
