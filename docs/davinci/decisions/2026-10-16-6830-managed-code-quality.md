# Managed Code Quality runner

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Preserve GitHub Code Quality and move its managed analysis to the existing
`blacksmith-32vcpu-ubuntu-2404` runner label. This is a repository setting;
the generated workflow at `dynamic/github-code-quality/codeql` is not an
editable workflow in this repository. The runner setting was applied on
October 16. Reading the configuration back confirmed that the languages,
schedule, enabled state, and AI analysis were unchanged. Fresh-run routing and
timings are acceptance evidence tracked in [PR #9198](https://github.com/ubugeeei-prod/vize/pull/9198).

## Evidence and configuration

On October 16, [run 38861276871](https://github.com/ubugeeei-prod/vize/actions/runs/38861276871)
took 6m23s from start to final update. Its JavaScript/TypeScript job took 6m19s,
including a 5m52s analysis step; Python took 59s. Both used `ubuntu-latest`.
These timings are separate from the required Check workflow.

The existing setup is enabled for `javascript-typescript` and `python`, with
`runner_type: standard`, a weekly schedule, and `ai_findings_option: on_push`.
Keep those languages, analysis behavior, and schedule. Removing Python would
not shorten the observed critical path.

GitHub explicitly supports labeled self-hosted runners for managed Code Quality.
Blacksmith documents this label and provisions runners through GitHub's
organization runner registration API. That supports using the existing provider;
successful routing of this managed workflow must still be observed.
[GitHub runner support](https://docs.github.com/en/code-security/reference/code-quality/codeql-detection),
[Blacksmith runner labels and registration](https://docs.blacksmith.sh/blacksmith-runners/overview).

The minimal update to `PATCH /repos/ubugeeei-prod/vize/code-quality/setup` is:

```json
{
  "runner_type": "labeled",
  "runner_label": "blacksmith-32vcpu-ubuntu-2404"
}
```

Use API version `2026-03-10`. GitHub's update schema accepts a partial body with
at least one supported field, and this payload changes only the runner fields.
The schema supplies no defaults for omitted languages or AI options. The API
does not expose `schedule` as an update field. Read back the configuration to
verify that all other settings were retained; the documented schema alone is
not evidence of the resulting server state.
[Code Quality API](https://docs.github.com/en/rest/code-quality/code-quality?apiVersion=2026-03-10),
[Official OpenAPI schema](https://github.com/github/rest-api-description/blob/main/descriptions/api.github.com/api.github.com.json).

## Acceptance and rollback

Save the complete current setup before applying the update. Afterward, require
`state`, `languages`, `schedule`, and `ai_findings_option` to match that snapshot,
and require the two runner fields to match the payload. Wait for any
configuration-validation run and a fresh PR-triggered run to finish. Inspect
actual job labels, successful analysis, queue time, and total elapsed time;
changing the setting alone does not establish the five-minute target.

If routing or analysis fails, restore `runner_type: standard` and clear
`runner_label`, then verify the remaining settings against the snapshot.
Preserve all analysis languages and findings. Record the observed run and
configuration results in PR #9198 and issue #6830 before reporting the
optimization as verified.
