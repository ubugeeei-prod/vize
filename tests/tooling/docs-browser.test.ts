import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { writeFakeCommand } from "./support/fake-command.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("docs build prepares its browser before previews and checks the rendered site", () => {
  const packageJson: unknown = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "docs", "package.json"), "utf8"),
  );
  assert(typeof packageJson === "object" && packageJson !== null && "scripts" in packageJson);
  const scripts = packageJson.scripts;
  assert(typeof scripts === "object" && scripts !== null);
  function script(name: string): string {
    assert(
      typeof scripts === "object" && scripts !== null && name in scripts,
      `Missing Docs script ${name}`,
    );
    const command: unknown = Reflect.get(scripts, name);
    assert(typeof command === "string", `Docs script ${name} must be a string`);
    return command;
  }

  assert.deepEqual(script("build").split(" && "), [
    "node ./scripts/ensure-browser.ts",
    "pnpm generate:ui-previews",
    "pnpm generate:reference",
    "vp build",
    "node ./scripts/finish-build.ts",
  ]);
  const finish = fs.readFileSync(path.join(repoRoot, "docs", "scripts", "finish-build.ts"), "utf8");
  assert.match(
    finish,
    /finishSteps = \["generate:og-images", "check:ui-docs", "check:heading-links"\] as const/,
  );
  const entrypoints = Object.fromEntries(
    [...finish.matchAll(/^  "([^"]+)": "([^"]+)",$/gm)].map((match) => [match[1], match[2]]),
  );
  assert.deepEqual(Object.keys(entrypoints), [
    "generate:og-images",
    "check:ui-docs",
    "check:heading-links",
  ]);
  for (const [name, entrypoint] of Object.entries(entrypoints))
    assert.equal(
      script(name),
      `node ${entrypoint}`,
      `${name}: execute the declared Node entrypoint`,
    );
  assert.equal(script("generate:ui-previews"), "node ./scripts/build-ui-previews.ts");
  assert.equal(script("generate:reference"), "node ../npm/ui/scripts/generate-reference-docs.ts");
  assert.equal(script("check:ui-docs"), "node ./previews/ui/check-site.ts");
  assert.equal(script("check:heading-links"), "node ./scripts/check-heading-hash.ts");
  assert.equal(script("generate:og-images"), "node ./scripts/generate-og-images.ts");
});

test("docs finish runs all Node phases concurrently and preserves complete failure output", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "docs-finish-phases-"));
  const docs = path.join(tempDir, "docs");
  const bin = path.join(tempDir, "bin");
  const pmLog = path.join(tempDir, "package-manager.log");
  const phases = [
    ["generate:og-images", "scripts/generate-og-images.ts"],
    ["check:ui-docs", "previews/ui/check-site.ts"],
    ["check:heading-links", "scripts/check-heading-hash.ts"],
  ];
  try {
    fs.mkdirSync(path.join(docs, "scripts"), { recursive: true });
    fs.mkdirSync(bin);
    writeFileSync(path.join(tempDir, "package.json"), JSON.stringify({ type: "module" }));
    fs.copyFileSync(
      path.join(repoRoot, "docs/scripts/finish-build.ts"),
      path.join(docs, "scripts/finish-build.ts"),
    );
    for (const command of ["pnpm", "npm", "yarn", "vp"])
      writeFakeCommand(
        bin,
        command,
        `require("node:fs").writeFileSync(${JSON.stringify(pmLog)}, ${JSON.stringify(command)}); process.exit(99);`,
      );
    for (const [index, [name, entrypoint]] of phases.entries()) {
      const file = path.join(docs, entrypoint);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      writeFileSync(
        file,
        `import { writeFileSync, readdirSync, existsSync } from "node:fs";
import { join } from "node:path";
import { setTimeout } from "node:timers/promises";
const receipt = { name: ${JSON.stringify(name)}, cwd: process.cwd(), execPath: process.execPath };
const output = process.env.VIZE_DOCS_FINISH_TEST_OUTPUT;
writeFileSync(join(output, "${index}-started.json"), JSON.stringify(receipt));
const deadline = Date.now() + 4000;
while (readdirSync(output).filter((name) => name.endsWith("-started.json")).length !== 3) {
  if (Date.now() >= deadline) throw new Error("All three phases must start before any can finish");
  await setTimeout(10);
}
const uiFinished = join(output, "1-finished.json");
while (${index} !== 1 && !existsSync(uiFinished)) {
  if (Date.now() >= deadline) throw new Error("The UI phase must finish first");
  await setTimeout(10);
}
writeFileSync(join(output, "${index}-finished.json"), JSON.stringify({
  ...receipt, uiFinishedBeforeThisPhase: existsSync(uiFinished),
}));
process.stdout.write("stdout-complete\\n");
process.stderr.write("stderr-complete\\n");
if (process.env.VIZE_DOCS_FINISH_TEST_FAILURE === "${index}") process.exit(7);
`,
      );
    }
    for (const failure of ["", "1"]) {
      const output = path.join(tempDir, failure === "" ? "success" : "failure");
      fs.mkdirSync(output);
      const result = spawnSync(process.execPath, [path.join(docs, "scripts/finish-build.ts")], {
        encoding: "utf8",
        timeout: 10_000,
        env: {
          ...process.env,
          PATH: [bin, path.dirname(process.execPath), process.env.PATH ?? ""].join(path.delimiter),
          VIZE_DOCS_FINISH_TEST_OUTPUT: output,
          VIZE_DOCS_FINISH_TEST_FAILURE: failure,
        },
      });
      assert.equal(fs.existsSync(pmLog), false, "finish phases must never run a package manager");
      assert.equal(result.error, undefined);
      assert.equal(result.status, failure === "" ? 0 : 1, result.stderr);
      assert.deepEqual(fs.readdirSync(output).sort(), [
        "0-finished.json",
        "0-started.json",
        "1-finished.json",
        "1-started.json",
        "2-finished.json",
        "2-started.json",
      ]);
      for (const [index, [name]] of phases.entries()) {
        const packet: unknown = JSON.parse(
          fs.readFileSync(path.join(output, `${index}-finished.json`), "utf8"),
        );
        assert.deepEqual(packet, {
          name,
          cwd: fs.realpathSync(docs),
          execPath: process.execPath,
          uiFinishedBeforeThisPhase: index !== 1,
        });
      }
      assert.deepEqual(
        result.stdout.split("\n").sort(),
        [...phases.map(([name]) => `[${name}] stdout-complete`), ""].sort(),
      );
      assert.deepEqual(
        result.stderr.split("\n").sort(),
        [
          ...phases.map(([name]) => `[${name}] stderr-complete`),
          ...(failure === "1" ? ["Docs build failed: check:ui-docs"] : []),
          "",
        ].sort(),
      );
      if (failure === "1") {
        assert.match(result.stderr, /^Docs build failed: check:ui-docs$/m);
      } else assert.doesNotMatch(result.stderr, /Docs build failed:/);
    }
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("docs browser helper reuses an existing browser path without invoking Playwright install", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "docs-browser-helper-"));
  const binDir = path.join(tempDir, "bin");
  const fakeBrowserPath = path.join(tempDir, "chromium");
  const playwrightLogPath = path.join(tempDir, "playwright.log");
  const helperPath = path.join(repoRoot, "docs", "scripts", "ensure-browser.ts");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(fakeBrowserPath, "");
    writeFakeCommand(
      binDir,
      "playwright",
      [
        "require('node:fs').appendFileSync(",
        `  ${JSON.stringify(playwrightLogPath)},`,
        "  process.argv.slice(2).join(' ') + '\\n',",
        ");",
        "process.exit(0);",
      ].join("\n"),
    );

    const result = spawnSync("node", [helperPath], {
      cwd: path.join(repoRoot, "docs"),
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        PUPPETEER_EXECUTABLE_PATH: fakeBrowserPath,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, new RegExp(`Using browser at ${fakeBrowserPath}`));
    assert.equal(fs.existsSync(playwrightLogPath), false);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
