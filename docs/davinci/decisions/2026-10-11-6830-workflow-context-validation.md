# Validate workflow contexts before starting qualification

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Problem and decision

An invalid `${{ runner.temp }}` in a job-level `env` mapping can cause GitHub
to reject a workflow with zero jobs. The PR check rollup then lacks the intended
qualification instead of reporting its failure. The existing Zizmor security
audit and parsed YAML inventory tests did not validate context availability.

Use the official [actionlint](https://github.com/rhysd/actionlint) checker in
the existing required `check-js` job, before its original native runtime
setup. Its event selectors, source trust boundaries, permissions, timeout,
commands and original qualification remain unchanged. A workflow that gates
only itself can be rejected with zero jobs and miss the PR rollup; using the
independent required Check job also catches invalid Zizmor and optional
workflows. Invalid Check itself leaves its required checks unsatisfied.
The original Zizmor workflow remains byte-exact. No Rust build, package
installation or additional job is introduced.

The composite action downloads the official Linux amd64 release 1.7.12,
SHA-256 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`,
verifies it before extraction and prints its version. The existing pinned
GitHub Script Node runtime runs the standard-library driver; no ambient
system Node or new package installation is assumed. Three existing Blacksmith
labels are explicitly declared. Diagnostic ignore configuration is refused.

## Whole checker and platform compatibility

Check every actual `.github/workflows/*.yml` and `*.yaml` file. Preserve all
syntax, expression, context, local-action and reusable-workflow diagnostics.
No original shell or Python check is removed; this new syntax pass does not
add actionlint's optional external ShellCheck/Pyflakes integrations.

Released actionlint 1.7.12 predates the supported `concurrency.queue` key.
[GitHub's official syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#concurrency)
permits literal `single` and `max`, and rejects `max` together with
`cancel-in-progress: true`. Keep the real deployment queue behavior.

The driver accepts only the checker's exact unsupported `queue` key diagnostic
at its physical concurrency mapping, with a recognized literal value. For
`max`, cancellation must be absent or literal `false`; an expression is not
silently assumed safe. Invalid queue values, incompatible cancellation and
every other syntax/context diagnostic still fail. Flow-form or unfamiliar
queue spellings are conservatively refused until explicitly qualified.
Original diagnostics and the narrow compatibility observation are retained.

Eight actual binary controls run before the whole source scan: valid step
`runner.temp`, invalid job `runner.temp` including embedded interpolation,
invalid top-level step context, valid queue maximum, invalid maximum plus
cancellation, invalid queue value, and a valid maximum that still retains an
independent invalid job context. The checker parses source only; the fixture
`run` commands are never executed.

## Evidence and delivery

The first local full scan of main `1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71`
reported 126 undeclared runner-label diagnostics and one supported queue key
diagnostic, with no other native checker diagnostics. After declaring exactly
the existing labels and validating that supported queue, all 71 actual
workflows and all eight original controls pass on the authenticated official
Darwin arm64 release. Its archive SHA-256 is
`aba9ced2dee8d27fecca3dc7feb1a7f9a52caefa1eb46f3271ea66b6e0e6953f`.
This local pass is not hosted Linux acceptance.

The first hosted Linux scan passes all 72 current workflows and eight actual
controls. The original matrix-runner inventory then exposed a suffix-match
bug: it treated the checker's `self-hosted-runner: labels` configuration as a
matrix `runner` key. Require the complete YAML key boundary while preserving
all real `host` and `runner` entries, including inline matrix mappings. The
actual pinned configuration remains the regression input; no runner checks
or checker diagnostics are waived.

Each hosted scan retains all control sources, original diagnostics, refusals,
supported-queue observations, workflow file hashes, checker binary hash,
source SHA and Node version under `target/workflow-syntax`. The validator
composite uploads this packet inside required `check-js` after success or
failure. Changed source during
the scan refuses qualification. A checker crash, unexpected status, incomplete
JSON or control disagreement also fails the job.

TODO: verify exact-head hosted Linux validation and real refusal controls,
preserve all existing ordinary and protected checks through actual merge,
and remove the narrow compatibility adapter after a pinned released checker
supports GitHub's queue key. Do not treat a missing or zero-job qualification
as successful CI.
