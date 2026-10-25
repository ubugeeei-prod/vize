import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../tests/differential/build-receipt.ts";
import { captureCssHover } from "./css-hover-visual-capture.mjs";
import { verifyCssHoverSourceCustody } from "./css-hover-source-custody.mjs";
import {
  baselineRevision,
  fixtureSource,
  vscodeVersion,
  workspaceSettings,
} from "./css-hover-visual-contract.mjs";
import {
  createPackagedHostEnvironment,
  createPackagedHostInstallArgs,
  createPackagedHostLaunchArgs,
  createRealHostEnvironment,
  resolveInstalledExtensionPath,
  runVSCodeCommandWithTimeout,
} from "./packaged-host-contract.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const sourceExtensionPath = path.join(root, "editors/vscode");
const { downloadAndUnzipVSCode, runVSCodeCommand } = createRequire(
  path.join(sourceExtensionPath, "package.json"),
)("@vscode/test-electron");
const { chromium } = createRequire(path.join(root, "tests/package.json"))("@playwright/test");
const destination = path.resolve(
  process.env.VIZE_CSS_VISUAL_OUTPUT ?? path.join(root, "output/playwright/css-hover"),
);
fs.mkdirSync(destination, { recursive: true });
assert.ok(
  !fs.existsSync(path.join(destination, "phase.json")),
  "capture requires a fresh output directory",
);
const write = (name, value) =>
  fs.writeFileSync(path.join(destination, name), `${JSON.stringify(value, null, 2)}\n`);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const identity = expectedBuildIdentity(root);
const receiptPath = path.join(root, `${identity.binaryPath}.differential-build.json`);
const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
validateBuildReceipt(receipt, identity);
let sourceCustody;
try {
  sourceCustody = verifyCssHoverSourceCustody(root, identity.sourceRevision);
  assert.equal(sourceCustody.proofKind, "current-checkout");
  assert.equal(sourceCustody.historicalBaselineRevision, baselineRevision);
  write("source-custody.json", sourceCustody);
} catch (error) {
  write("source-custody-error.json", { message: error.message, stack: error.stack });
  throw error;
}
const serverPath = fs.realpathSync(path.join(root, identity.binaryPath));
const actualVersion = execFileSync(serverPath, ["--version"], { encoding: "utf8" }).trim();
assert.equal(actualVersion, identity.cliVersion);
fs.copyFileSync(receiptPath, path.join(destination, "build-receipt.json"));
const vsixPath = path.join(sourceExtensionPath, "dist/vize.vsix");
fs.copyFileSync(vsixPath, path.join(destination, "vize.vsix"));
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-"));
const extensionsPath = path.join(profile, "ext");
const userDataPath = path.join(profile, "user");
const workspacePath = path.join(profile, "workspace");
fs.mkdirSync(path.join(workspacePath, ".vscode"), { recursive: true });
fs.mkdirSync(path.join(userDataPath, "User"), { recursive: true });
fs.writeFileSync(path.join(workspacePath, "Colors.vue"), fixtureSource);
fs.writeFileSync(
  path.join(workspacePath, "package.json"),
  '{"private":true,"name":"css-hover-capture"}\n',
);
const settingsBytes = `${JSON.stringify(workspaceSettings(serverPath), null, 2)}\n`;
fs.writeFileSync(path.join(workspacePath, ".vscode/settings.json"), settingsBytes);
fs.writeFileSync(path.join(userDataPath, "User/settings.json"), settingsBytes);
fs.writeFileSync(path.join(destination, "Colors.vue"), fixtureSource);
fs.writeFileSync(path.join(destination, "settings.json"), settingsBytes);
fs.writeFileSync(
  path.join(destination, "workspace-package.json"),
  fs.readFileSync(path.join(workspacePath, "package.json")),
);

const portServer = net.createServer();
await new Promise((resolve) => portServer.listen(0, "127.0.0.1", resolve));
const port = portServer.address().port;
await new Promise((resolve) => portServer.close(resolve));
const output = ({ stdout, stderr }) => {
  if (stdout) process.stdout.write(stdout);
  if (stderr) process.stderr.write(stderr);
};
let capture;
try {
  const vscodeExecutable = await downloadAndUnzipVSCode({ version: vscodeVersion });
  const vscodePackage = path.join(path.dirname(vscodeExecutable), "resources/app/package.json");
  const vscodeProduct = path.join(path.dirname(vscodeExecutable), "resources/app/product.json");
  // The visual workflow runs Linux Electron, whose app metadata accompanies
  // this exact executable; refuse a host with a substituted version.
  assert.equal(JSON.parse(fs.readFileSync(vscodePackage, "utf8")).version, vscodeVersion);
  fs.copyFileSync(vscodePackage, path.join(destination, "vscode-package.json"));
  fs.copyFileSync(vscodeProduct, path.join(destination, "vscode-product.json"));
  output(
    await runVSCodeCommandWithTimeout(
      runVSCodeCommand,
      createPackagedHostInstallArgs({ extensionsPath, userDataPath, vsixPath }),
      {
        environment: createPackagedHostEnvironment(process.env),
        timeoutMs: 120_000,
        version: vscodeVersion,
      },
    ),
  );
  const installedExtensionPath = resolveInstalledExtensionPath(extensionsPath, "ubugeeei.vize");
  const installedFiles = [];
  function recordFiles(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const full = path.join(directory, entry.name);
      if (entry.isDirectory()) recordFiles(full);
      else if (entry.isFile())
        installedFiles.push({
          path: path.relative(installedExtensionPath, full),
          sha256: hash(fs.readFileSync(full)),
          bytes: fs.statSync(full).size,
        });
    }
  }
  recordFiles(installedExtensionPath);
  fs.copyFileSync(
    path.join(installedExtensionPath, "package.json"),
    path.join(destination, "installed-package.json"),
  );
  const environment = createPackagedHostEnvironment({
    ...createRealHostEnvironment({
      extensionsPath,
      processEnvironment: process.env,
      resultPath: path.join(destination, "unused-result.json"),
      serverPath,
      sourceExtensionPath,
    }),
    VIZE_CSS_VISUAL_OUTPUT: destination,
  });
  const args = [
    ...createPackagedHostLaunchArgs({
      extensionsPath,
      userDataPath,
      installedExtensionPath,
      workspacePath,
      extensionTestsPath: path.join(sourceExtensionPath, "test/suite/css-hover-visual.cjs"),
    }),
    `--remote-debugging-port=${port}`,
    "--disable-gpu",
    "--disable-dev-shm-usage",
  ];
  write("identity.json", {
    source: identity,
    actualCliVersion: actualVersion,
    sourceTree: git("rev-parse", "HEAD^{tree}"),
    baselineRevision,
    productionChanges: sourceCustody.productionChanges,
    sourceCustody,
    vscodeVersion,
    vscodeExecutable: { path: vscodeExecutable, sha256: hash(fs.readFileSync(vscodeExecutable)) },
    vsix: { sha256: hash(fs.readFileSync(vsixPath)), bytes: fs.statSync(vsixPath).size },
    installedExtensionPath,
    installedFiles,
    launchArgs: args,
    hostEnvironment: {
      VIZE_TEST_ENABLE_HOST_COMMANDS: environment.VIZE_TEST_ENABLE_HOST_COMMANDS,
      VIZE_TEST_SERVER_PATH: environment.VIZE_TEST_SERVER_PATH,
      nodeOptionsRemoved: environment.NODE_OPTIONS === undefined,
    },
    latencyMeaning:
      "Actual public API command wall time in this one fresh packaged editor session; no startup, budget, or admission claim.",
  });
  capture = captureCssHover({ chromium, port, destination });
  // Attach immediately so a renderer failure cannot become an unhandled rejection
  // while the extension host writes its independent failure evidence.
  const observed = capture.then(
    () => ({ ok: true }),
    (error) => {
      write("capture-error.json", {
        message: error.message,
        cause: String(error.cause ?? ""),
        stack: error.stack,
      });
      return { error };
    },
  );
  const result = await runVSCodeCommandWithTimeout(runVSCodeCommand, args, {
    environment,
    stdio: ["ignore", "inherit", "inherit"],
    timeoutMs: 300_000,
    version: vscodeVersion,
  });
  output(result);
  const captured = await observed;
  if (captured.error) throw captured.error;
  assert.ok(
    fs.existsSync(path.join(destination, "complete.json")),
    "host completion evidence is required",
  );
} catch (error) {
  write("runner-error.json", {
    message: error.message,
    stack: error.stack,
    cause: String(error.cause ?? ""),
  });
  throw error;
} finally {
  for (const [input, name] of [
    [path.join(userDataPath, "logs"), "vscode-logs"],
    [path.join(workspacePath, "node_modules/.vize"), "native-logs"],
  ]) {
    if (fs.existsSync(input)) fs.cpSync(input, path.join(destination, name), { recursive: true });
  }
  fs.rmSync(profile, { force: true, recursive: true });
}
