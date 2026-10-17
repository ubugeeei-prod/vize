import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { vrtHarness } from "../../tools/support/release/public_acceptance/vrt_fixtures.ts";

const project = fileURLToPath(new URL("../../tsconfig.tooling-migration.json", import.meta.url));
const checker = fileURLToPath(
  new URL("../../tools/support/typescript/check-project.ts", import.meta.url),
);

void test("the installed VRT source closure uses real strict Browser types", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const vrtProject = join(root, "tsconfig.public-vrt.json");
  const config = JSON.parse(readFileSync(vrtProject, "utf8"));
  const comparePaths = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);
  const additionalModules = [
    "installed.ts",
    "vrt_literal.ts",
    "vrt_retained.ts",
    "vrt_retained_scan.ts",
    "vrt_secrets.ts",
    "vrt_service_announcement.ts",
    "vrt_service_process.ts",
    "vrt_trusted_color_build.ts",
    "vrt_trusted_color_capture.ts",
    "vrt_trusted_colors.ts",
    "vrt_trusted_failures.ts",
    "vrt_trusted_gallery.ts",
    "vrt_trusted_network.ts",
    "vrt_trusted_observation.ts",
    "vrt_trusted_report_isolation.ts",
    "vrt_trusted_surface.ts",
    "vrt_trusted_ui.ts",
    "vrt_tunnel.ts",
    "vrt_tunnel_metadata.ts",
    "vrt_tunnel_process.ts",
    "vrt_tunnel_setup.ts",
  ];
  const additionalTargets = [
    "tests/tooling/fixtures/release/vrt-log-barrier/observer.fixture.ts",
    "tests/tooling/release/release-public-gallery-readiness.test.ts",
    "tests/tooling/release/release-public-quick-tunnel.test.ts",
    "tests/tooling/release/release-public-trusted-vrt-color.test.ts",
    "tests/tooling/release/release-public-trusted-vrt.test.ts",
    "tests/tooling/release/release-public-vrt-announcement.test.ts",
    "tests/tooling/release/release-public-vrt-literal-inputs.test.ts",
    "tests/tooling/release/release-public-vrt-log-barrier.test.ts",
    "tests/tooling/release/release-public-vrt-tunnel-inputs.test.ts",
    "tests/tooling/release/release-public-vrt-upload-refusal.test.ts",
  ];
  const files = [
    ...[...vrtHarness, ...additionalModules].map(
      (file) => `tools/support/release/public_acceptance/${file}`,
    ),
    ...additionalTargets,
  ].sort(comparePaths);
  assert.equal(files.length, 39);
  assert.equal(new Set(files).size, files.length);
  assert.deepEqual([...config.files].sort(comparePaths), files);
  const require = createRequire(new URL("../../docs/package.json", import.meta.url));
  const manifest = realpathSync(require.resolve("playwright/package.json"));
  assert.equal(JSON.parse(readFileSync(manifest, "utf8")).version, "1.62.1");
  const whole = spawnSync(process.execPath, [checker, vrtProject], {
    encoding: "utf8",
    shell: false,
  });
  assert.equal(whole.error, undefined);
  assert.equal(whole.status, 0, whole.stdout + whole.stderr);
  const normalize = (file: string) => file.replaceAll("\\", "/");
  const directory = normalize(join(root, "tools/support/release/public_acceptance"));
  const closure = whole.stdout
    .split(/\r?\n/u)
    .filter((file) => normalize(file).startsWith(directory + "/") && file.endsWith(".ts"))
    .map(normalize)
    .sort();
  assert.deepEqual(
    closure,
    [
      ...files.filter((file) => file.startsWith("tools/support/release/public_acceptance/")),
      "tools/support/release/public_acceptance/a11y.ts",
      "tools/support/release/public_acceptance/http.ts",
      "tools/support/release/public_acceptance/inputs.ts",
      "tools/support/release/public_acceptance/plan.ts",
    ]
      .map((file) => normalize(join(root, file)))
      .sort(),
  );
  const temporary = mkdtempSync(join(root, ".vize-vrt-types-"));
  try {
    const witness = join(temporary, "witness.ts");
    const witnessConfig = join(temporary, "tsconfig.json");
    writeFileSync(join(temporary, "package.json"), JSON.stringify({ type: "module" }));
    writeFileSync(
      witnessConfig,
      JSON.stringify({ extends: vrtProject, files: [witness], include: [] }),
    );
    const api = JSON.stringify(
      normalize(join(root, "tools/support/release/public_acceptance/vrt_api.ts")),
    );
    const prefix = `import type { Browser } from 'playwright';\nimport { observeVrtApi } from ${api};\ndeclare const browser: Browser;\ndeclare const fixture: Parameters<typeof observeVrtApi>[1];\ndeclare const evidence: Parameters<typeof observeVrtApi>[2];\n`;
    const check = (source: string) => {
      writeFileSync(witness, prefix + source);
      const result = spawnSync(process.execPath, [checker, witnessConfig], {
        encoding: "utf8",
        shell: false,
      });
      assert.equal(result.error, undefined);
      return result;
    };
    const valid = check("void observeVrtApi(browser, fixture, evidence);\n");
    assert.equal(valid.status, 0, valid.stdout + valid.stderr);
    const wrongBrowser = check("void observeVrtApi({}, fixture, evidence);\n");
    assert.notEqual(wrongBrowser.status, 0);
    assert.match(wrongBrowser.stdout + wrongBrowser.stderr, /TS2740/u);
    const wrongViewport = check(
      "void browser.newPage({ viewport: { width: 'invalid', height: 180 } });\n",
    );
    assert.notEqual(wrongViewport.status, 0);
    assert.match(wrongViewport.stdout + wrongViewport.stderr, /TS2322/u);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

void test("the native tooling checker enforces strict erasable TypeScript on its actual project", () => {
  const owned = spawnSync(process.execPath, [checker], { encoding: "utf8", shell: false });
  assert.equal(owned.error, undefined);
  assert.equal(owned.status, 0, owned.stdout + owned.stderr);
  const directory = mkdtempSync(join(tmpdir(), "vize-tooling-types-"));
  const source = join(directory, "witness.ts");
  const config = join(directory, "tsconfig.json");
  writeFileSync(join(directory, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(
    config,
    JSON.stringify({
      extends: project,
      compilerOptions: { types: [] },
      files: [source],
      include: [],
    }),
  );
  const check = (text: string) => {
    writeFileSync(source, text);
    const result = spawnSync(process.execPath, [checker, config], {
      encoding: "utf8",
      shell: false,
    });
    assert.equal(result.error, undefined);
    return result;
  };
  try {
    const correct = check("export const count: number = 1;\n");
    assert.equal(correct.status, 0, correct.stdout + correct.stderr);
    for (const [text, diagnostic] of [
      ['export const count: number = "incorrect";\n', /TS2322/],
      ["export function identity(value) { return value; }\n", /TS7006/],
      ["export enum Level { First }\n", /TS1294/],
    ] as const) {
      const invalid = check(text);
      assert.notEqual(invalid.status, 0);
      assert.match(invalid.stdout + invalid.stderr, diagnostic);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
