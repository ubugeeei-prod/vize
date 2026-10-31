import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { sourceSdk } from "../../tools/benchmarks/scripts/canon-component-casing-observe.ts";

test("casing source adapter uses the complete current declarations and preserves the published provider", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-casing-sdk-"));
  try {
    const provider = path.join(directory, "provider");
    const published = path.join(provider, "node_modules/vize");
    fs.mkdirSync(published, { recursive: true });
    const publishedManifest = '{"name":"vize","version":"0.602.0","untouched":true}\n';
    const publishedHelper = "export interface HistoricalProviderOnly {}\n";
    fs.writeFileSync(path.join(published, "package.json"), publishedManifest);
    fs.writeFileSync(path.join(published, "content-mapper-helpers.d.ts"), publishedHelper);
    const project = path.join(directory, "source");
    fs.mkdirSync(project);
    const mapper = path.join(directory, "selected-source-mapper");
    const custody = sourceSdk(project, provider, mapper);
    const sourceRoot = fileURLToPath(new URL("../../", import.meta.url));
    const revision = execFileSync("git", ["rev-parse", "HEAD"], { cwd: sourceRoot })
      .toString()
      .trim();
    assert.equal(custody.source, revision);
    assert.deepEqual(
      custody.declarations.map(({ file }) => file),
      [
        "virtual-ts-globals.d.ts",
        "virtual-ts-patterns.d.ts",
        "virtual-ts.d.ts",
        "virtual-ts.d.mts",
        "virtual-ts.d.cts",
        "content-mapper-helpers.d.ts",
        "content-mapper-helpers.d.mts",
        "content-mapper-helpers.d.cts",
        "generated/content-mapper-types.d.ts",
      ],
    );
    for (const { file, source, installed } of custody.declarations) {
      assert.deepEqual(
        fs.readFileSync(path.join(project, "node_modules/vize", file)),
        execFileSync("git", ["show", `${revision}:npm/cli/${file}`], { cwd: sourceRoot }),
      );
      assert.equal(fs.lstatSync(installed.path).isSymbolicLink(), false);
      assert.deepEqual(
        { bytes: installed.bytes, sha256: installed.sha256 },
        { bytes: source.bytes, sha256: source.sha256 },
      );
    }
    const expected = JSON.parse(
      fs.readFileSync(path.join(sourceRoot, "npm/cli/package.json"), "utf8"),
    ) as { version: string; typescript: { contentMapper: { exec: string[] } } };
    expected.version = `0.0.0-source-${revision}`;
    expected.typescript.contentMapper.exec = [mapper, "content-mapper"];
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(project, "node_modules/vize/package.json"), "utf8")),
      expected,
    );
    assert.equal(fs.readFileSync(path.join(published, "package.json"), "utf8"), publishedManifest);
    assert.equal(
      fs.readFileSync(path.join(published, "content-mapper-helpers.d.ts"), "utf8"),
      publishedHelper,
    );
    assert.throws(() => sourceSdk(project, provider, mapper), { code: "EEXIST" });
    assert.equal(
      fs.readFileSync(path.join(published, "content-mapper-helpers.d.ts"), "utf8"),
      publishedHelper,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
