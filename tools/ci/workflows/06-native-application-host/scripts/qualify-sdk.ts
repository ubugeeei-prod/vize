import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

interface SimulatorRuntime {
  readonly identifier: string;
  readonly isAvailable: boolean;
  readonly name: string;
  readonly version: string;
}

function command(program: string, args: readonly string[]): string {
  const result = spawnSync(program, args, { encoding: "utf8", timeout: 60_000 });
  if (result.error || result.status !== 0) {
    throw new Error(`Cannot qualify ${program}: ${result.error?.message ?? result.stderr.trim()}`);
  }
  return result.stdout.trim();
}

const diagnostics: { readonly code: string; readonly message: string }[] = [];
let xcode = "";
let sdk = "";
let runtimes: SimulatorRuntime[] = [];
try {
  if (process.platform !== "darwin") throw new Error("the first host requires a macOS build host");
  xcode = command("xcodebuild", ["-version"]);
  sdk = command("xcrun", ["--sdk", "iphonesimulator", "--show-sdk-path"]);
  const discovered = JSON.parse(command("xcrun", ["simctl", "list", "runtimes", "--json"])) as {
    readonly runtimes: SimulatorRuntime[];
  };
  runtimes = discovered.runtimes
    .filter(
      (runtime) =>
        runtime.isAvailable &&
        runtime.identifier.startsWith("com.apple.CoreSimulator.SimRuntime.iOS-"),
    )
    .map(({ identifier, isAvailable, name, version }) => ({
      identifier,
      isAvailable,
      name,
      version,
    }));
  if (runtimes.length === 0) throw new Error("no available iOS simulator runtime is installed");
} catch (error) {
  diagnostics.push({
    code: "native-target-sdk-unavailable",
    message: error instanceof Error ? error.message : "native target SDK qualification failed",
  });
}

const receipt = {
  format: "vize.native-host.sdk-qualification",
  formatVersion: 1,
  source: process.env.GITHUB_SHA ?? null,
  platform: process.platform,
  architecture: process.arch,
  available: diagnostics.length === 0,
  xcode,
  sdk,
  runtimes,
  diagnostics,
};
const output = path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "native-application-sdk.json");
writeFileSync(output, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(JSON.stringify({ ...receipt, receipt: output }));
if (!receipt.available) process.exitCode = 1;
