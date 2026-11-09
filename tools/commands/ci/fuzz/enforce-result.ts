import { isEntrypoint } from "../../../support/command-runtime.ts";

const supportedEvents = new Set(["pull_request", "merge_group", "schedule", "workflow_dispatch"]);
const supportedOutcomes = new Set(["success", "failure", "cancelled", "skipped"]);
const blockingEvents = new Set(["merge_group", "schedule", "workflow_dispatch"]);

function label(value: string): string {
  return value === "" ? "empty" : value;
}

export function enforceFuzzResult(arguments_: readonly string[]): number {
  if (arguments_.length !== 3) {
    console.error(
      "Usage: node tools/commands/ci/fuzz/enforce-result.ts <event-name> <target> <outcome>",
    );
    return 1;
  }
  const [eventName, target, outcome] = arguments_ as [string, string, string];
  if (!supportedEvents.has(eventName)) {
    console.error(`Unsupported fuzz event: ${label(eventName)}`);
    return 1;
  }
  if (!supportedOutcomes.has(outcome)) {
    console.error(`Unsupported fuzz outcome: ${label(outcome)}`);
    return 1;
  }
  if (outcome === "success") {
    console.log(`Fuzz target ${target} completed successfully.`);
    return 0;
  }

  const message = `Fuzz target ${target} finished with ${outcome} on ${eventName}.`;
  if (blockingEvents.has(eventName)) {
    console.error(`::error::${message} Release evidence must be green.`);
    return 1;
  }
  console.error(`::warning::${message} Pull-request fuzzing is advisory.`);
  return 0;
}

if (isEntrypoint(import.meta.url)) process.exitCode = enforceFuzzResult(process.argv.slice(2));
