---
title: Doctor Diagnostics
---

# Doctor Diagnostics

Use `vize doctor` to investigate application health across Vue components and script modules.
It reports problems such as dependency-injection mismatches, circular imports, server/client
boundaries, and cross-component prop contracts, with the source locations that explain each finding.
Start with a terminal report, inspect the affected code, and rerun after making a change.

Install the CLI using [Getting Started](../getting-started.md), or find the other commands in the
[CLI guide](cli.md#doctor).

## Run Your First Report

From your project directory, analyze the source directory:

```bash
vize doctor src
```

Replace `src` with your application's source directory. Run `vize doctor` without a path to scan
the current workspace, or pass several directories to include shared application code:

```bash
vize doctor src packages/shared
```

The command reads Vue SFCs and script modules without changing your files. The text report starts
with the workspace, a health score out of 100, and counts of errors, warnings, and notices. Each
finding includes a diagnostic code, message, and source location; use those locations to inspect
the affected relationship before changing the code. Rerun the same command to check the result.

A report with no findings means the current analyzers found no problems in the selected sources.
It does not establish that every application behavior has been checked. The command exits with
`1` for a certain or high-confidence error, and `2` when discovery, analysis, or report output fails.
Warnings and lower-confidence findings do not block by default. The full policy is explained in
[Scoring and Exit Status](#scoring-and-exit-status).

## Save a Report for CI

Use JSON when another tool needs to consume the findings:

```bash
vize doctor src --format json > doctor-report.json
```

The JSON report retains diagnostic identities, evidence, and scoring metadata. Redirection saves
the report without changing the command's exit status. Keep the status when Doctor owns your CI
gate; use `--exit-zero` only when another system owns the blocking policy. Read the
[Automation Contract](#automation-contract) before building a report consumer.

## Narrow the Findings

Add filters when you need to focus on a category, severity, rule, or changed source. Filtering
selects findings after analysis; it does not reduce which analyzers run. Start with the complete
report so that relevant cross-component context remains visible, then use the options below.

`vize doctor` analyzes the application once, then projects the immutable report into the view a
person, CI gate, editor, or AI integration needs. Filters never change which analyzers run or mutate
the evidence graph. They select findings after analysis and produce a newly scored report.

## Filter Algebra

Repeat a filter to accept multiple values. Category, severity, and confidence also accept
comma-separated values. Values within one dimension are ORed, while populated dimensions are ANDed:

```bash
vize doctor src \
  --category correctness,security \
  --severity error \
  --confidence certain,high \
  --rule 'VIZE_DOCTOR_CF_*' \
  --path 'packages/**/src/*.vue' \
  --changed-file 'packages/account/**'
```

This example retains a finding only when every populated dimension matches. An empty dimension
matches everything. Repeating `--rule` therefore broadens the rule dimension; adding `--severity`
narrows the result across dimensions.

| Option           | Matches                                                                                                 |
| ---------------- | ------------------------------------------------------------------------------------------------------- |
| `--category`     | `correctness`, `accessibility`, `performance`, `maintainability`, `security`, or `production-readiness` |
| `--severity`     | `error`, `warning`, or `notice`                                                                         |
| `--confidence`   | `certain`, `high`, `medium`, or `low`                                                                   |
| `--target`       | Analyzer-supplied target identifier                                                                     |
| `--rule`         | Stable diagnostic code                                                                                  |
| `--path`         | Primary workspace-relative source path only                                                             |
| `--route`        | Analyzer-supplied route identifier                                                                      |
| `--environment`  | Analyzer-supplied runtime environment                                                                   |
| `--package`      | Analyzer-supplied workspace package                                                                     |
| `--changed-file` | Any affected source in the complete evidence graph                                                      |

Identifier and path dimensions use case-sensitive shell globs. `*`, `?`, character classes, and
recursive `**` are supported. Paths are compared with `/` separators on every platform. Invalid or
empty patterns fail closed before analysis output is written, with exit status `2` and a diagnostic
that names the invalid dimension.

## Changed-File Semantics

`--changed-file` is intentionally broader than `--path`. A finding matches when the pattern reaches
any of these inputs:

- its primary or related source location;
- a nested evidence location;
- a source edit in the proposed fix; or
- a provenance invalidation input that would require the analysis to be recomputed.

This preserves findings whose visible primary location did not change but whose conclusion depends
on a shared component, generated contract, or related declaration that did. It also makes the filter
suitable for pull-request annotations without discarding cross-file causality.

## Scoring and Exit Status

The filtered report is reconstructed through the normal deterministic ranking and scoring path.
Counts, category health, overall score, and blocking-error state describe only visible findings.
Consequently, hiding the last blocking finding changes the command exit status from `1` to `0`.

| Status | Meaning                                                                  |
| -----: | ------------------------------------------------------------------------ |
|    `0` | The visible report has no certain or high-confidence error               |
|    `1` | At least one visible finding is a blocking error                         |
|    `2` | Discovery, analysis, filter compilation, serialization, or output failed |

Use `--exit-zero` when the report is informative and another system owns policy. It changes the
process status only; it does not rewrite the report's blocking-error state.

## Automation Contract

Use `--format json` for CI, editors, dashboards, and provider-neutral AI adapters. The JSON document
has explicit format and scoring versions, stable rule codes, normalized paths, confidence, impact,
evidence, fix availability, and analysis provenance. Consumers should filter through the CLI or the
`vize_doctor::DoctorFilterSpec` library contract instead of deleting JSON findings themselves; doing
so guarantees that derived health metadata and exit policy stay consistent.

Filters are compiled once. Matching a report performs a single pass over findings and only traverses
the extra evidence graph when `--changed-file` is populated. The reporter receives the resulting
immutable report, so text, JSON, SARIF, TUI, and future AI vendors share identical selection
semantics.
