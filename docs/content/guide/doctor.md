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

The JSON report keeps each finding's code, locations, and score. Redirecting it to a file does not
change the exit status, so the same command can still fail the CI job.

For GitHub code scanning or another SARIF consumer, write SARIF instead:

```bash
vize doctor src --format sarif > doctor.sarif
```

> [!TIP]
> Use `--exit-zero` only when another system decides whether the build fails. Read the
> [Automation Contract](#automation-contract) before building your own report consumer.

## Narrow the Findings

Add filters when you need to focus on a category, severity, rule, or changed source. Filters pick
findings after the whole application has been analyzed, so cross-component findings are never
lost; the score is then recomputed for what remains. Start with the complete report, then narrow it.

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

Identifier and path filters use case-sensitive shell globs (`*`, `?`, `[abc]`, and recursive `**`),
with `/` as the separator on every platform. An invalid or empty pattern stops the command with exit
status `2` and names the filter that is wrong.

## Changed-File Semantics

`--changed-file` is broader than `--path`. A finding matches when a changed file is:

- where the finding is reported, or one of its related locations;
- a file the finding's reasoning goes through;
- a file its suggested fix would edit; or
- an input whose change would change the finding.

So a finding in an unchanged component still shows up when the shared component or declaration it
depends on changed. That makes `--changed-file` the right filter for pull-request annotations.

## Scoring and Exit Status

Counts, category health, the overall score, and the exit status describe only the findings left
after filtering. Filtering out the last blocking finding therefore changes the exit status from `1`
to `0`.

| Status | Meaning                                                                  |
| -----: | ------------------------------------------------------------------------ |
|    `0` | The visible report has no certain or high-confidence error               |
|    `1` | At least one visible finding is a blocking error                         |
|    `2` | Discovery, analysis, filter compilation, serialization, or output failed |

Use `--exit-zero` when the report is informative and another system owns policy. It changes the
process status only; it does not rewrite the report's blocking-error state.

## Automation Contract

Use `--format json` for CI, editors, dashboards, and AI tools. The document carries format and scoring
versions, stable rule codes, normalized paths, confidence, impact, the reasoning behind each finding,
and whether a fix is available. Filter with the CLI options above (or
`vize_doctor::DoctorFilterSpec` from Rust) rather than deleting findings from the JSON yourself, so
the score and exit status stay consistent with what you show. Text, JSON, and SARIF output all
apply the same filters.
