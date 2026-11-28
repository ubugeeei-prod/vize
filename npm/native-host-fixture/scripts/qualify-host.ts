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

function currentSource(): string {
  const changes = command("git", [
    "-C",
    packageRoot,
    "status",
    "--porcelain",
    "--untracked-files=no",
  ]);
  if (changes !== "")
    throw new Error("Native qualification requires a clean tracked source checkout.");
  return command("git", ["-C", packageRoot, "rev-parse", "HEAD"]);
}

function verifiedApplication(): ApplicationBuild {
  const receipt = JSON.parse(
    readFileSync(path.join(packageRoot, ".build/application-build.json"), "utf8"),
  ) as ApplicationBuild;
  const source = currentSource();
  const hash = createHash("sha256")
    .update(readFileSync(path.join(ios, "Resources/application.js")))
    .digest("hex");
  if (
    receipt.format !== "vize.native-host.application-build" ||
    receipt.formatVersion !== 1 ||
    receipt.source !== source ||
    receipt.sha256 !== hash ||
    receipt.compilerSourceVerified !== true ||
    receipt.trackedSourceClean !== true
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

const packetResults = path.join(output, "packet-tests.xml");
command(
  "swift",
  [
    "test",
    "--package-path",
    ios,
    "--scratch-path",
    path.join(output, "swift-build"),
    "--disable-swift-testing",
    "--xunit-output",
    packetResults,
  ],
  "packet-tests.log",
);
const packets = readFileSync(packetResults, "utf8");
const packetTestCount = [...packets.matchAll(/<testcase\b/g)].length;
if (packetTestCount !== 10 || /<(?:failure|error|skipped)\b/.test(packets)) {
  throw new Error("Native packet qualification requires ten passing XCTest cases without skips.");
}
let ownedDevice = "";
let selectedDeviceName = "";
const refusedTypes: { readonly identifier: string; readonly message: string }[] = [];
let cleanupFailed = false;
try {
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
      const created = result.stdout.trim();
      if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(created))
        throw new Error("The simulator create command did not return an owned device identifier.");
      ownedDevice = created;
      selectedDeviceName = name;
      break;
    }
    refusedTypes.push({ identifier, message: result.error?.message ?? result.stderr.trim() });
  }
  writeFileSync(
    path.join(output, "device-types.log"),
    `${JSON.stringify(refusedTypes, null, 2)}\n`,
  );
  if (ownedDevice === "")
    throw new Error("The selected native SDK has no available compatible phone type.");
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
  const interaction = JSON.parse(
    command("xcrun", [
      "xcresulttool",
      "get",
      "test-results",
      "summary",
      "--path",
      resultBundle,
      "--compact",
    ]),
  ) as {
    readonly result: string;
    readonly totalTestCount: number;
    readonly passedTests: number;
    readonly failedTests: number;
    readonly skippedTests: number;
    readonly expectedFailures: number;
  };
  writeFileSync(
    path.join(output, "interaction-summary.json"),
    `${JSON.stringify(interaction, null, 2)}\n`,
  );
  if (
    interaction.result !== "Passed" ||
    interaction.totalTestCount !== 2 ||
    interaction.passedTests !== 2 ||
    interaction.failedTests !== 0 ||
    interaction.skippedTests !== 0 ||
    interaction.expectedFailures !== 0
  ) {
    throw new Error(
      "Native interaction qualification requires two passing UI cases without skips.",
    );
  }
  const application = path.join(
    derived,
    "Build/Products/Debug-iphonesimulator/VizeNativeHostFixture.app",
  );
  const bytes = totalBytes(application);
  const maximumBytes = 5 * 1024 * 1024;
  if (bytes > maximumBytes)
    throw new Error("The first native host application exceeds its binary contribution budget.");
  if (currentSource() !== applicationBuild.source) {
    throw new Error("The native qualification source changed during the run.");
  }
  const finalBuild = verifiedApplication();
  const testedHash = createHash("sha256")
    .update(readFileSync(path.join(application, "application.js")))
    .digest("hex");
  if (
    finalBuild.sha256 !== applicationBuild.sha256 ||
    finalBuild.compilerSha256 !== applicationBuild.compilerSha256 ||
    finalBuild.compilerBuildToken !== applicationBuild.compilerBuildToken ||
    testedHash !== applicationBuild.sha256
  ) {
    throw new Error("The tested native application does not match the admitted build.");
  }
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
    packetTests: packetTestCount,
    nativeInteractionTests: interaction.passedTests,
    resultBundle,
  };
  writeFileSync(path.join(output, "qualification.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt));
} finally {
  // Only this invocation's new simulator is shut down and removed.
  if (ownedDevice !== "") {
    const shutdown = spawnSync("xcrun", ["simctl", "shutdown", ownedDevice], { encoding: "utf8" });
    const deletion = spawnSync("xcrun", ["simctl", "delete", ownedDevice], { encoding: "utf8" });
    writeFileSync(
      path.join(output, "simulator-cleanup.log"),
      `${shutdown.stdout ?? ""}${shutdown.stderr ?? ""}${deletion.stdout ?? ""}${deletion.stderr ?? ""}`,
    );
    cleanupFailed = deletion.error !== undefined || deletion.status !== 0;
  }
}
if (cleanupFailed) throw new Error("The invocation-owned native simulator could not be removed.");
