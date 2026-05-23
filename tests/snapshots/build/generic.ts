import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { VIZE_BIN, requireVizeBin } from "../../_helpers/apps.ts";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const FIXTURE_DIR = path.resolve(__dirname, "../../_fixtures/_projects/generic-build");
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");

describe("generic build snapshots (compiler)", () => {
  before(() => {
    requireVizeBin();
  });

  it("snapshots selected generated outputs exactly", () => {
    const outDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-generic-build-"));

    try {
      const stdout = execFileSync(
        VIZE_BIN,
        ["build", "src/**/*.vue", "-o", outDir, "--continue-on-error"],
        {
          cwd: FIXTURE_DIR,
          timeout: 120_000,
          maxBuffer: 100 * 1024 * 1024,
        },
      ).toString();
      console.log(stdout);

      const jsFiles = fs
        .readdirSync(outDir, { recursive: true })
        .map((entry) => String(entry))
        .filter((entry) => entry.endsWith(".js"))
        .sort();

      assert.deepEqual(jsFiles, [
        "DirectiveBuiltins.js",
        "FixturePanel.js",
        "NormalScriptBindings.js",
        "SlotOutlet.js",
      ]);

      const snapshotOutputs = [
        ["DirectiveBuiltins.js", "generic-directive-builtins"],
        ["FixturePanel.js", "generic-fixture-panel"],
        ["NormalScriptBindings.js", "generic-normal-script-bindings"],
        ["SlotOutlet.js", "generic-slot-outlet"],
      ] as const;

      for (const [file, snapshotName] of snapshotOutputs) {
        const content = fs.readFileSync(path.join(outDir, file), "utf-8");
        assertParsesAsModule(content, file);
        assertSnapshot(SNAPSHOT_DIR, snapshotName, content);
      }
    } finally {
      fs.rmSync(outDir, { recursive: true, force: true });
    }
  });
});
