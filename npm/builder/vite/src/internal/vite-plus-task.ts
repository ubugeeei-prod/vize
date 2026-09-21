import { runTask } from "../vite-plus/runner.ts";

try {
  process.exitCode = await runTask(process.argv.slice(2));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode =
    error && typeof error === "object" && "exitCode" in error && typeof error.exitCode === "number"
      ? error.exitCode
      : 1;
}
