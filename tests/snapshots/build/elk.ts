import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { elkApp, VIZE_BIN } from "../../_helpers/apps.ts";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const app = elkApp;

describe(`${app.name} build (compiler)`, () => {
  before(() => {
    if (!fs.existsSync(VIZE_BIN)) {
      console.log(`Skipping: vize binary not found at ${VIZE_BIN}`);
      process.exit(0);
    }
  });

  it("vize build compiles without errors", () => {
    const cwd = app.check!.cwd; // same patterns as check
    const patterns = app.check!.patterns.map((p) => `'${p}'`).join(" ");
    const outDir = path.join(__dirname, "__snapshots__", `${app.name}-build-output`);
    fs.rmSync(outDir, { recursive: true, force: true });

    const cmd = `${VIZE_BIN} build ${patterns} -o '${outDir}' --continue-on-error`;
    console.log(`Running: ${cmd}`);

    const stdout = execSync(cmd, {
      cwd,
      timeout: 120_000,
      maxBuffer: 100 * 1024 * 1024,
    }).toString();

    console.log(stdout);

    // Verify output directory has .js files
    assert.ok(fs.existsSync(outDir), "output directory should exist");
    const jsFiles = fs
      .readdirSync(outDir, { recursive: true })
      .filter((f) => String(f).endsWith(".js"));
    console.log(`Generated ${jsFiles.length} JS files`);
    assert.ok(jsFiles.length > 0, "should produce .js output files");
  });

  it("compiled output is valid JavaScript", () => {
    const outDir = path.join(__dirname, "__snapshots__", `${app.name}-build-output`);
    if (!fs.existsSync(outDir)) {
      assert.fail("output directory does not exist - run build test first");
    }

    const jsFiles = fs
      .readdirSync(outDir, { recursive: true })
      .filter((f) => String(f).endsWith(".js"))
      .slice(0, 10); // Check first 10 files

    for (const entry of jsFiles) {
      const file = String(entry);
      const filePath = path.join(outDir, file);
      const content = fs.readFileSync(filePath, "utf-8");
      assertParsesAsModule(content, file);
      console.log(`Valid: ${file}`);
    }
  });
});
