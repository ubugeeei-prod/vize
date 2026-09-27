# Workflow security audit selection (2026-09-27)

Tracking issue: [#6866](https://github.com/ubugeeei-prod/vize/issues/6866).

The Zizmor workflow receives every pull request. Forks and authors outside
GitHub's `OWNER`, `MEMBER` and `COLLABORATOR` associations always scan.
A same-repository maintainer PR skips the scan only when the read-only
GitHub API proves its complete, unchanged file list has no `.github/`
changes. Both current and previous filenames are checked, so moving a
workflow out of `.github/` still scans.

The selector uses pinned `actions/github-script` with trusted inline code,
`pull-requests: read` and no checkout. It never executes candidate code or
uses `pull_request_target`. Failed API requests, changed heads, empty or
incomplete file lists and GitHub's 3,000-file limit all retain the scan.
The scanner also runs when its planning job fails or omits its result.

Published releases, explicit dispatches and the weekly scheduled audit
always scan; `.github/` pushes retain the existing path filter. Fork scans
use console mode so their read-only tokens suffice; this mode
fails on findings. Same-repository scans retain Advanced Security output
and the existing action, tool version, online audits and severity policy.
The title-policy workflow remains active for every PR.

The release event audits published release tags. It does not replace the
existing exact-SHA release preflight or make a new prepublication gate.

References: [GitHub pull request file API](https://docs.github.com/en/rest/pulls/pulls#list-pull-requests-files)
documents the file limit; [zizmor-action](https://github.com/zizmorcore/zizmor-action#usage-without-github-advanced-security)
documents console mode and its finding-based failure behavior.
