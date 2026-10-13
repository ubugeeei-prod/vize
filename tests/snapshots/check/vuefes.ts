import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { vuefesApp, CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { retainAppCheckPacket } from "../../_helpers/app-check-packet.ts";
import { prepareNuxtCheckFixture } from "../../_helpers/nuxt-check-prepare.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = vuefesApp;

describe(`${app.name} check (type checker)`, () => {
  before(() => {
    requireVizeAndCorsaBins();
    prepareNuxtCheckFixture({
      name: app.name,
      sourceDir: app.check!.cwd,
      env: app.env,
      installTimeoutMs: 300_000,
    });
  });

  it("vize check does not crash and snapshot matches", () => {
    const checkConfig = app.check!;
    const patterns = checkConfig.patterns.map((p) => `'${p}'`).join(" ");
    const cmd = `${VIZE_BIN} check ${patterns} --format json --quiet --corsa-path '${CORSA_BIN}'`;
    console.log(`Running: ${cmd}`);

    let stdout: string;
    try {
      stdout = execSync(cmd, {
        cwd: checkConfig.cwd,
        timeout: 120_000,
        maxBuffer: 100 * 1024 * 1024,
      }).toString();
    } catch (e: any) {
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize check crashed (exit code ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    const parsed = JSON.parse(stdout);
    retainAppCheckPacket({
      name: app.name,
      stdout,
      parsed,
      cwd: checkConfig.cwd,
      command: cmd,
      argv: [
        VIZE_BIN,
        "check",
        ...checkConfig.patterns,
        "--format",
        "json",
        "--quiet",
        "--corsa-path",
        CORSA_BIN,
      ],
    });
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");
    const diagnostics: string[] = parsed.files.flatMap(
      (file: { diagnostics: string[] }) => file.diagnostics,
    );
    assert.equal(diagnostics.length, parsed.errorCount, "unmapped diagnostics found");
    // The feature flag module registers this augmentation in Nuxt's
    // `prepare:types` hook, so the prepared fixture must resolve it.
    assert.deepEqual(
      diagnostics.filter((diagnostic) => diagnostic.includes("'vfFeatures'")),
      [],
      "import.meta.vfFeatures must resolve through the prepared Nuxt declarations",
    );

    const { programs: _programs, ...snapshotOutput } = parsed;
    const prettyOutput =
      JSON.stringify(snapshotOutput, null, 2).replaceAll(checkConfig.cwd, "<cwd>") + "\n";
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
