import path from "node:path";
import { fileURLToPath } from "node:url";

const outcomes = new Set(["success", "failure", "cancelled", "skipped"]);
const events = new Set(["pull_request", "schedule", "workflow_dispatch"]);
const releaseEvents = new Set(["schedule", "workflow_dispatch"]);

export function fuzzResultPolicy(eventName, outcome) {
  if (!events.has(eventName)) {
    throw new Error(`Unsupported fuzz event: ${eventName || "empty"}`);
  }
  if (!outcomes.has(outcome)) {
    throw new Error(`Unsupported fuzz outcome: ${outcome || "empty"}`);
  }
  const unsuccessful = outcome !== "success";
  return {
    unsuccessful,
    releaseBlocking: unsuccessful && releaseEvents.has(eventName),
  };
}

export function reportFuzzResult(eventName, target, outcome) {
  const policy = fuzzResultPolicy(eventName, outcome);
  if (!policy.unsuccessful) {
    console.log(`Fuzz target ${target} completed successfully.`);
    return 0;
  }

  const message = `Fuzz target ${target} finished with ${outcome} on ${eventName}.`;
  if (policy.releaseBlocking) {
    console.error(`::error::${message} Release evidence must be green.`);
    return 1;
  }
  console.warn(`::warning::${message} Pull-request fuzzing is advisory.`);
  return 0;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const [eventName, target, outcome, extra] = process.argv.slice(2);
  try {
    if (!eventName || !target || !outcome || extra != null) {
      throw new Error(
        "Usage: rust-script tools/commands/ci/fuzz/enforce-result.rs <event-name> <target> <outcome>",
      );
    }
    process.exitCode = reportFuzzResult(eventName, target, outcome);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
