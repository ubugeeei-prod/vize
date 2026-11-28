import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const ios = path.join(packageRoot, "ios");
const output = mkdtempSync(
  path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-native-first-host-"),
);

function command(program: string, args: readonly string[], log?: string): string {
  const result = spawnSync(program, args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  const text = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  if (log) writeFileSync(path.join(output, log), text);
  if (result.error || result.status !== 0)
    throw new Error(
      `Native host qualification failed in ${program}: ${result.error?.message ?? text.slice(-12000)}`,
    );
  return result.stdout.trim();
}

function totalBytes(directory: string): number {
  return readdirSync(directory, { withFileTypes: true }).reduce((sum, entry) => {
    const file = path.join(directory, entry.name);
    return sum + (entry.isDirectory() ? totalBytes(file) : statSync(file).size);
  }, 0);
}

interface ApplicationBuild {
  readonly format: string;
  readonly formatVersion: number;
  readonly source: string;
  readonly trackedSourceClean: boolean;
  readonly compilerSourceVerified: boolean;
  readonly compilerSha256: string;
  readonly compilerBuildToken: string;
  readonly sha256: string;
}

function verifiedApplication(): ApplicationBuild {
  const receipt = JSON.parse(
    readFileSync(path.join(packageRoot, ".build/application-build.json"), "utf8"),
  ) as ApplicationBuild;
  const source = command("git", ["-C", packageRoot, "rev-parse", "HEAD"]);
  const hash = createHash("sha256")
    .update(readFileSync(path.join(ios, "Resources/application.js")))
    .digest("hex");
  if (
    receipt.format !== "vize.native-host.application-build" ||
    receipt.formatVersion !== 1 ||
    receipt.source !== source ||
    receipt.sha256 !== hash ||
    receipt.compilerSourceVerified !== true ||
    (process.env.GITHUB_ACTIONS === "true" && receipt.trackedSourceClean !== true)
  ) {
    throw new Error(
      "Native qualification requires the application built by this checkout's compiler.",
    );
  }
  return receipt;
}

const applicationBuild = verifiedApplication();

interface Runtime {
  readonly identifier: string;
  readonly version: string;
  readonly isAvailable: boolean;
  readonly supportedDeviceTypes?: readonly { readonly identifier: string; readonly name: string }[];
}
interface Device {
  readonly udid: string;
  readonly name: string;
  readonly isAvailable: boolean;
  readonly deviceTypeIdentifier?: string;
}
const sdk = command("xcrun", ["--sdk", "iphonesimulator", "--show-sdk-version"]);
const runtimeList = JSON.parse(command("xcrun", ["simctl", "list", "runtimes", "--json"])) as {
  readonly runtimes: Runtime[];
};
const selectedRuntime = runtimeList.runtimes.find(
  ({ identifier, version, isAvailable }) =>
    isAvailable &&
    identifier.startsWith("com.apple.CoreSimulator.SimRuntime.iOS-") &&
    version.split(".").slice(0, 2).join(".") === sdk.split(".").slice(0, 2).join("."),
);
if (!selectedRuntime)
  throw new Error("The first native host needs a simulator runtime matching the selected SDK.");
const discovered = JSON.parse(
  command("xcrun", ["simctl", "list", "devices", "available", "--json"]),
) as { readonly devices: Readonly<Record<string, readonly Device[]>> };
const deviceTypes = JSON.parse(command("xcrun", ["simctl", "list", "devicetypes", "--json"])) as {
  readonly devicetypes: readonly { readonly name: string; readonly identifier: string }[];
};
const candidates = new Map<string, string>();
for (const device of discovered.devices[selectedRuntime.identifier] ?? []) {
  if (!device.isAvailable || !device.name.startsWith("iPhone")) continue;
  const type =
    device.deviceTypeIdentifier ??
    deviceTypes.devicetypes.find(({ name }) => name === device.name)?.identifier;
  if (type) candidates.set(type, device.name);
}
for (const type of [...(selectedRuntime.supportedDeviceTypes ?? []), ...deviceTypes.devicetypes]) {
  if (type.name.startsWith("iPhone") && !candidates.has(type.identifier))
    candidates.set(type.identifier, type.name);
}

command(
  "swift",
  ["test", "--package-path", ios, "--scratch-path", path.join(output, "swift-build")],
  "packet-tests.log",
);
let ownedDevice = "";
let selectedDeviceName = "";
const refusedTypes: { readonly identifier: string; readonly message: string }[] = [];
for (const [identifier, name] of candidates) {
  const result = spawnSync(
    "xcrun",
    [
      "simctl",
      "create",
      `Vize First Host ${path.basename(output)}`,
      identifier,
      selectedRuntime.identifier,
    ],
    { encoding: "utf8" },
  );
  if (!result.error && result.status === 0) {
    ownedDevice = result.stdout.trim();
    selectedDeviceName = name;
    break;
  }
  refusedTypes.push({ identifier, message: result.error?.message ?? result.stderr.trim() });
}
writeFileSync(path.join(output, "device-types.log"), `${JSON.stringify(refusedTypes, null, 2)}\n`);
if (!/^[0-9a-f-]{36}$/i.test(ownedDevice))
  throw new Error("The selected native SDK has no available compatible phone type.");
try {
  command("xcrun", ["simctl", "bootstatus", ownedDevice, "-b"], "simulator-boot.log");
  const derived = path.join(output, "DerivedData");
  const resultBundle = path.join(output, "interaction.xcresult");
  command(
    "xcodebuild",
    [
      "test",
      "-project",
      path.join(ios, "VizeNativeHostFixture.xcodeproj"),
      "-scheme",
      "VizeNativeHostFixture",
      "-configuration",
      "Debug",
      "-destination",
      `platform=iOS Simulator,id=${ownedDevice}`,
      "-derivedDataPath",
      derived,
      "-resultBundlePath",
      resultBundle,
      "CODE_SIGNING_ALLOWED=NO",
      "-parallel-testing-enabled",
      "NO",
    ],
    "interaction-tests.log",
  );
  const application = path.join(
    derived,
    "Build/Products/Debug-iphonesimulator/VizeNativeHostFixture.app",
  );
  const bytes = totalBytes(application);
  const maximumBytes = 5 * 1024 * 1024;
  if (bytes > maximumBytes)
    throw new Error("The first native host application exceeds its binary contribution budget.");
  const receipt = {
    format: "vize.native-host.simulator-qualification",
    formatVersion: 1,
    source: applicationBuild.source,
    trackedSourceClean: applicationBuild.trackedSourceClean,
    compilerSha256: applicationBuild.compilerSha256,
    compilerBuildToken: applicationBuild.compilerBuildToken,
    applicationSha256: applicationBuild.sha256,
    sdk,
    runtime: { identifier: selectedRuntime.identifier, version: selectedRuntime.version },
    simulator: selectedDeviceName,
    applicationBytes: bytes,
    maximumBytes,
    packetTests: "passed",
    nativeInteractionTests: "passed",
    resultBundle,
  };
  writeFileSync(path.join(output, "qualification.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt));
} finally {
  // Only this invocation's new simulator is shut down and removed.
  spawnSync("xcrun", ["simctl", "shutdown", ownedDevice], { encoding: "utf8" });
  spawnSync("xcrun", ["simctl", "delete", ownedDevice], { encoding: "utf8" });
}
