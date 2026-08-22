import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { vuefesApp, CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = vuefesApp;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

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
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.equal(parsed.fileCount, 81, "authored transitive sources must remain reported");
    assert.equal(parsed.errorCount, 47, "VueFes authored diagnostic baseline drifted");

    const authored = parsed.files.filter((file: { file: string }) => !file.file.endsWith(".vue"));
    assert.deepEqual(
      authored.map((file: { file: string }) => file.file),
      [
        "app/stores/animation.ts",
        "i18n/en/goods.ts",
        "i18n/en/related-events.ts",
        "i18n/en/speakers.ts",
        "i18n/goods.ts",
        "i18n/ja/goods.ts",
        "i18n/ja/related-events.ts",
        "i18n/ja/speakers.ts",
        "i18n/related-events.ts",
        "i18n/speaker.ts",
      ],
      "explicit Vue inputs must keep their exact authored TypeScript dependency closure",
    );
    const diagnosticsByFile = Object.fromEntries(
      authored.map((file: { file: string; diagnostics: string[] }) => [
        file.file,
        file.diagnostics,
      ]),
    );
    assert.deepEqual(
      Object.entries(diagnosticsByFile)
        .filter(([, diagnostics]) => (diagnostics as string[]).length > 0)
        .map(([file]) => file),
      ["app/stores/animation.ts", "i18n/en/speakers.ts", "i18n/ja/speakers.ts"],
      "every authored diagnostic must map to one classified source",
    );
    assert.deepEqual(diagnosticsByFile["app/stores/animation.ts"], [
      "error:82:28 [TS7006] Parameter 'value' implicitly has an 'any' type.",
    ]);
    const speakerDiagnostics = [
      [34, 35],
      [57, 58],
      [73, 74],
      [100, 101],
      [118, 119],
      [140, 141],
    ].flatMap(([featureLine, nameLine]) => [
      `error:${featureLine}:28 [TS2339] Property 'vfFeatures' does not exist on type 'ImportMeta'.`,
      `error:${nameLine}:31 [TS2339] Property 'vfFeatures' does not exist on type 'ImportMeta'.`,
    ]);
    assert.deepEqual(diagnosticsByFile["i18n/en/speakers.ts"], speakerDiagnostics);
    assert.deepEqual(diagnosticsByFile["i18n/ja/speakers.ts"], speakerDiagnostics);
    const authoredErrors = authored.reduce(
      (count: number, file: { diagnostics: string[] }) => count + file.diagnostics.length,
      0,
    );
    const existingVueErrors = parsed.files
      .filter((file: { file: string }) => file.file.endsWith(".vue"))
      .reduce(
        (count: number, file: { diagnostics: string[] }) => count + file.diagnostics.length,
        0,
      );
    assert.equal(authoredErrors, 25, "all newly authored diagnostics must remain classified");
    assert.equal(existingVueErrors, 22, "the previous Vue diagnostic baseline must remain intact");
    assert.equal(
      authoredErrors + existingVueErrors,
      parsed.errorCount,
      "unmapped diagnostics found",
    );

    const { programs: _programs, ...snapshotOutput } = parsed;
    const prettyOutput =
      JSON.stringify(snapshotOutput, null, 2).replaceAll(checkConfig.cwd, "<cwd>") + "\n";
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
