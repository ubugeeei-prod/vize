import {
  main,
  observers,
} from "../../../../.github/actions/test-js-packages-with-history/native-fault-evidence.ts";

const [mode, ...command] = process.argv.slice(2);
function unavailable() {
  const error = new Error("typed observation gap");
  error.name = "PermissionError";
  throw error;
}

observers.sourceSnapshot = unavailable;
observers.privateGuest =
  mode === "guest" ? unavailable : () => ({ privateFirecrackerGuest: false });
observers.runnerResources = () => ({ syntheticResources: true });
if (mode === "descendants") observers.observeDescendants = unavailable;
if (mode === "gdb") observers.diagnoseCore = unavailable;
if (mode.startsWith("task-details")) observers.observeDescendants = () => {};
if (mode === "task-details-unavailable") observers.taskDetails = unavailable;
process.exitCode = await main(["--", ...command], "linux");
