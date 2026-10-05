import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import vm from "node:vm";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { prepareScenarioContracts } from "../../editors/vscode/test/prepare-scenario-contracts.mjs";
import { runPackagedExtensionHost } from "../../editors/vscode/test/packaged-host-contract.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));

await test("the client starts with both complete original contract-hover inputs already on disk", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-contract-startup-"));
  const workspacePath = path.join(directory, "workspace");
  const extensionsPath = path.join(directory, "extensions");
  const vsixPath = path.join(directory, "vize.vsix");
  const source = path.join(root, "editors/vscode/test/suite/real-scenario-expected.cjs");
  // Only read the existing input constants; this does not emulate a provider.
  const module = { exports: {} };
  vm.runInNewContext(fs.readFileSync(source, "utf8"), {
    module,
    require: (name: string) => {
      assert.equal(name, "vscode");
      return { Position: class {}, Range: class {}, DiagnosticSeverity: { Error: 0, Warning: 1 } };
    },
  });
  const expected = module.exports as {
    componentContractChildSource: string;
    componentContractHostSource: string;
  };
  const corpus = path.join(
    root,
    "tests/_fixtures/differential/lsp-regressions/contract-before-startup-8012",
  );
  const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "manifest.json"), "utf8")) as {
    cases: Array<{ file: string; fixture: string; bytes: number; sha256: string }>;
  };
  assert.deepEqual(
    manifest.cases.map((entry) => entry.file),
    ["src/ContractChild.vue", "src/ContractHost.vue"],
  );
  fs.mkdirSync(path.join(workspacePath, "src"), { recursive: true });
  const scenario = Buffer.from("unrelated scorecard source stays unchanged\n");
  fs.writeFileSync(path.join(workspacePath, "src/Scenario.vue"), scenario);
  fs.mkdirSync(extensionsPath);
  fs.writeFileSync(vsixPath, "");
  let started = 0;
  try {
    prepareScenarioContracts(workspacePath);
    await runPackagedExtensionHost(
      async (args) => {
        if (args.includes("--install-extension")) {
          const installed = path.join(extensionsPath, "ubugeeei.vize-0.432.0");
          fs.mkdirSync(installed);
          fs.writeFileSync(
            path.join(installed, "package.json"),
            JSON.stringify({ publisher: "ubugeeei", name: "vize" }),
          );
        } else {
          started += 1;
          for (const entry of manifest.cases) {
            const actual = fs.readFileSync(path.join(workspacePath, entry.file));
            const original = Buffer.from(
              entry.file.endsWith("Child.vue")
                ? expected.componentContractChildSource
                : expected.componentContractHostSource,
            );
            assert.deepEqual(actual, original);
            assert.deepEqual(actual, fs.readFileSync(path.join(corpus, entry.fixture)));
            assert.equal(actual.length, entry.bytes);
            assert.equal(crypto.createHash("sha256").update(actual).digest("hex"), entry.sha256);
          }
          assert.deepEqual(fs.readFileSync(path.join(workspacePath, "src/Scenario.vue")), scenario);
        }
        return { stdout: "", stderr: "" };
      },
      {
        extensionId: "ubugeeei.vize",
        extensionsPath,
        extensionTestsPath: path.join(root, "editors/vscode/test/suite/extension-host-real.cjs"),
        hostEnvironment: {},
        hostTimeoutMs: 1_000,
        installEnvironment: {},
        installTimeoutMs: 1_000,
        onOutput: () => {},
        userDataPath: path.join(directory, "profile"),
        vscodeVersion: "1.107.1",
        vsixPath,
        workspacePath,
      },
    );
    assert.equal(started, 1);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
