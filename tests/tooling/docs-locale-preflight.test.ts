import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFile, mkdir, mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import type { Browser } from "playwright";
import { repoRoot } from "./_helpers/moonbit.ts";

const sourceFiles = [
  "docs/theme/i18n/locale-switcher.js",
  "docs/theme/i18n/locale-selector.css",
  "docs/theme/i18n/navigation.js",
  "docs/theme/background.ts",
  "docs/scripts/locale-render-assertions.ts",
  "docs/scripts/locale-responsive-assertions.ts",
  "docs/scripts/locale-control-render.ts",
  "docs/scripts/verify-navigation-render.ts",
];

async function copySource(root: string, file: string) {
  const destination = path.join(root, file);
  await mkdir(path.dirname(destination), { recursive: true });
  await copyFile(path.join(repoRoot, file), destination);
}

for (const mode of ["missing source", "gitless checkout"] as const) {
  void test(`locale campaign retains its whole partial receipt for ${mode}`, async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "vize-docs-locale-preflight-"));
    try {
      for (const basename of [
        "locale-control-render",
        "locale-render-assertions",
        "locale-responsive-assertions",
        "theme-render-assertions",
      ]) {
        await copySource(root, `docs/scripts/${basename}.ts`);
      }
      const available = mode === "missing source" ? sourceFiles.slice(0, 1) : sourceFiles;
      for (const file of available) await copySource(root, file);
      if (mode === "gitless checkout") {
        const control = spawnSync("git", ["rev-parse", "HEAD"], {
          cwd: root,
          encoding: "utf8",
        });
        assert.notEqual(control.status, 0, "the real checkout has no Git identity");
        assert.match(control.stderr, /not a git repository/);
      }
      const { verifyLocaleRenderControls } = await import(
        pathToFileURL(path.join(root, "docs/scripts/locale-control-render.ts")).href
      );
      let browserCalls = 0;
      const browser = {
        newContext() {
          browserCalls++;
          throw new Error("preflight failure must not start a browser context");
        },
      } as unknown as Browser;
      const output = path.join(root, "output");
      let failure: unknown;
      await assert.rejects(
        verifyLocaleRenderControls(browser, "http://127.0.0.1:1", output),
        (error: unknown) => {
          failure = error;
          return error instanceof Error;
        },
      );
      assert.equal(browserCalls, 0);
      assert(failure instanceof Error);
      assert.match(
        failure.message,
        mode === "missing source" ? /ENOENT.*locale-selector\.css/ : /git rev-parse HEAD/,
      );
      const receipt = JSON.parse(
        await readFile(path.join(output, "locale-controls/receipt.json"), "utf8"),
      );
      assert.equal(receipt.completed, false);
      assert.equal(receipt.sourceHead, null);
      assert.equal(receipt.failure, failure.stack ?? String(failure));
      assert.deepEqual(receipt.cases, []);
      assert.deepEqual(receipt.workflow, {
        sha: process.env.GITHUB_SHA ?? null,
        run: process.env.GITHUB_RUN_ID ?? null,
        attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
      });
      const expected = await Promise.all(
        available.map(async (file) => {
          const bytes = await readFile(path.join(root, file));
          return {
            file,
            bytes: bytes.length,
            sha256: createHash("sha256").update(bytes).digest("hex"),
          };
        }),
      );
      assert.deepEqual(receipt.source, expected);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
}
