// Jobs whose planner can select only a skip notice take a hosted runner then,
// so they never hold a Blacksmith slot to print it (#6830). Their selected
// path keeps the reviewed maximum runner.
export const plannedBusyRunner = "blacksmith-32vcpu-ubuntu-2404";
export const plannedIdleRunner = "ubuntu-24.04";

export const plannedRunnerJobs: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  "01-check.yml": {
    "security-audit": "needs.iteration-plan.outputs.dependencies == 'true'",
    "node-engine-compat": "needs.iteration-plan.outputs.engines == 'true'",
    "check-vize-apps": "needs.iteration-plan.outputs.apps == 'true'",
  },
  "01-pr-source.yml": {
    "wasm-release-check":
      "contains(fromJSON(needs.pr-source-plan.outputs.rust-plan).packages, 'vize_vitrine')",
    "ci-selection-tests": "needs.pr-source-plan.outputs.tooling == 'true'",
    "pr-native-runtime": "matrix.lane != 'none'",
    "pr-playground-test": "needs.pr-source-plan.outputs.playground == 'true'",
  },
};

export function plannedRunner(condition: string) {
  return `\${{ ${condition} && '${plannedBusyRunner}' || '${plannedIdleRunner}' }}`;
}

/** The reviewed condition when `runs-on` is exactly this job's planned runner. */
export function plannedRunnerCondition(file: string, job: string, runsOn: unknown) {
  const condition = plannedRunnerJobs[file]?.[job];
  return condition !== undefined && runsOn === plannedRunner(condition) ? condition : undefined;
}
