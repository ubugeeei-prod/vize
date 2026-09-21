// Real VS Code (pinned in CI) with the packaged `editors/vscode/dist/vize.vsix`
// installed into an isolated profile; `vscode-suite.cjs` plays the scenario
// through VS Code's own provider commands inside the extension host, so
// every request is built by vscode-languageclient.
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import { repoRoot, suiteRoot, type Driver, type DriverContext } from "../support/context.ts";

const extensionRoot = path.join(repoRoot, "editors", "vscode");
const vscodeVersion = process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1";

async function run(context: DriverContext): Promise<void> {
  const { runVSCodeCommand } = createRequire(path.join(extensionRoot, "package.json"))(
    "@vscode/test-electron",
  ) as { runVSCodeCommand: (args: string[], options: object) => Promise<unknown> };
  const host = (await import(path.join(extensionRoot, "test", "packaged-host-contract.mjs"))) as {
    runPackagedExtensionHost: (run: unknown, options: object) => Promise<string>;
  };
  const scenarioFile = path.join(context.scratch, "scenario.resolved.json");
  fs.writeFileSync(scenarioFile, JSON.stringify(context.scenario));
  // The singleton socket lives in the user-data directory; keep it short (104-byte AF_UNIX cap).
  const profile = fs.mkdtempSync(path.join(path.dirname(context.workspace), "vsc-"));
  const extensionsPath = path.join(profile, "extensions");
  const shim = context.env.PATH!.split(path.delimiter)[0];
  // Configure the workspace the way a user does before opening it: what
  // `Vize: Enable Recommended Profile` writes, plus the formatting opt-in every
  // client makes. Changing settings after activation restarts the client, and
  // a document opened during a restart reaches two server sessions.
  const settingsFile = path.join(context.workspace, ".vscode", "settings.json");
  const settings = JSON.parse(fs.readFileSync(settingsFile, "utf8")) as Record<string, unknown>;
  fs.writeFileSync(
    settingsFile,
    `${JSON.stringify(
      {
        ...settings,
        "vize.enable": true,
        "vize.lint.enable": true,
        "vize.typecheck.enable": true,
        "vize.editor.enable": true,
        "vize.ecosystem.enable": true,
        "vize.formatting.enable": true,
        "vize.serverPath": path.join(shim, "vize"),
      },
      null,
      2,
    )}\n`,
  );
  const cachePath = path.join(extensionRoot, ".vscode-test");
  await host.runPackagedExtensionHost(
    (args: string[], options: object) => runVSCodeCommand(args, { ...options, cachePath }),
    {
      extensionId: "ubugeeei.vize",
      extensionsPath,
      extensionTestsPath: path.join(suiteRoot, "drivers", "vscode-suite.cjs"),
      hostEnvironment: {
        ...context.env,
        VIZE_TS45_SCENARIO: scenarioFile,
        VIZE_TS45_SERVER: path.join(shim, "vize"),
      },
      hostTimeoutMs: 600_000,
      installEnvironment: process.env,
      installTimeoutMs: 120_000,
      onOutput: ({ stdout, stderr }: { stdout?: string; stderr?: string }) => {
        if (stdout) process.stdout.write(stdout);
        if (stderr) process.stderr.write(stderr);
      },
      userDataPath: path.join(profile, "user-data"),
      vscodeVersion,
      vsixPath: path.join(extensionRoot, "dist", "vize.vsix"),
      workspacePath: context.workspace,
    },
  );
}

export const driver: Driver = {
  client: "VS Code",
  version: () => vscodeVersion,
  mode: "editor",
  run,
};
