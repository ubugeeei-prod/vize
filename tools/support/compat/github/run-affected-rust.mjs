import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export function rustCommand(plan, command) {
  if (
    plan?.schemaVersion !== 1 ||
    !["affected", "workspace"].includes(plan.scope) ||
    !Array.isArray(plan.packages) ||
    !plan.packages.length ||
    plan.packages.some(
      (name) => typeof name !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_-]*$/.test(name),
    ) ||
    new Set(plan.packages).size !== plan.packages.length
  ) {
    throw new Error("expected a nonempty, valid affected Rust plan");
  }
  const cargoArgs = plan.packages.flatMap((name) => ["--package", name]);
  if (JSON.stringify(plan.cargoArgs) !== JSON.stringify(cargoArgs)) {
    throw new Error("Rust plan package arguments disagree with its selected crates");
  }
  if (
    command[0] !== "cargo" ||
    !["clippy", "nextest", "test"].includes(command[1]) ||
    command.filter((argument) => argument === "@packages@").length !== 1 ||
    command.some((argument) => typeof argument !== "string" || argument.includes("\0"))
  ) {
    throw new Error("expected a Cargo check command with one @packages@ argument");
  }
  return command.flatMap((argument) => (argument === "@packages@" ? cargoArgs : [argument]));
}

export function main(argv = process.argv.slice(2)) {
  const [planPath, ...command] = argv;
  const [executable, ...args] = rustCommand(JSON.parse(readFileSync(planPath, "utf8")), command);
  const result = spawnSync(executable, args, { stdio: "inherit", shell: false });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
