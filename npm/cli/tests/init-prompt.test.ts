import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  exists,
  runInit,
  scriptedPrompt,
  temporaryProject,
  write,
  writeManifest,
} from "./init-support.ts";

function vitePlusProject(name: string): string {
  const root = temporaryProject(name);
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vp dev" },
    devDependencies: { "vite-plus": "^0.1.0" },
  });
  write(
    root,
    "vite.config.ts",
    `import { defineConfig } from "vite-plus";

export default defineConfig({
  plugins: [],
});
`,
  );
  write(root, "tsconfig.json", "{}\n");
  write(root, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
  return root;
}

const CHECKLIST_HEADER = `
Select the features to configure.
Type the numbers to toggle (space or comma separated), then press Enter.
`;

test("the checklist pre-ticks what detection found and reports what it cannot offer", async () => {
  const root = vitePlusProject("prompt-defaults");
  const prompt = scriptedPrompt(["", "y"]);

  const result = await runInit(root, [], prompt);

  assert.deepEqual(prompt.asked(), ["> ", "Apply this selection? [Y/n] "]);
  assert.equal(
    prompt.transcript(),
    `${CHECKLIST_HEADER}
  1. [x] oxlint plugin (vp lint reads the \`lint\` block in the Vite config)
  2. [x] vite plugin (@vizejs/vite-plugin)
  3. [x] fmt (vize fmt)
  4. [x] typecheck (vize check)
  5. [x] editor extension (.vscode/extensions.json recommendation)

`,
  );
  assert.deepEqual(
    result.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "configured"],
      ["bundler", "configured"],
      ["fmt", "configured"],
      ["typecheck", "configured"],
      ["editor", "configured"],
    ],
  );
});

test("toggling a number off removes exactly that feature from the plan", async () => {
  const root = vitePlusProject("prompt-toggle");
  // Toggle 4 (typecheck) and 5 (editor) off, accept, confirm.
  const prompt = scriptedPrompt(["4 5", "", ""]);

  const result = await runInit(root, ["--no-install"], prompt);

  assert.deepEqual(prompt.asked(), ["> ", "> ", "Apply this selection? [Y/n] "]);
  assert.deepEqual(
    result.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "configured"],
      ["bundler", "configured"],
      ["fmt", "configured"],
      ["typecheck", "skipped"],
      ["editor", "skipped"],
    ],
  );
  assert.deepEqual(result.plan?.addedScripts, ["vize:lint", "vize:fmt", "vize:fmt:fix"]);
  assert.equal(exists(root, ".vscode/extensions.json"), false);
  assert.deepEqual(result.commands, []);
});

test("an out-of-range answer is rejected without changing the selection", async () => {
  const root = vitePlusProject("prompt-invalid");
  const prompt = scriptedPrompt(["9", "", "y"]);

  await runInit(root, ["--no-install"], prompt);

  const checklist = `${CHECKLIST_HEADER}
  1. [x] oxlint plugin (vp lint reads the \`lint\` block in the Vite config)
  2. [x] vite plugin (@vizejs/vite-plugin)
  3. [x] fmt (vize fmt)
  4. [x] typecheck (vize check)
  5. [x] editor extension (.vscode/extensions.json recommendation)

`;
  // The rejected answer is reported and the checklist is re-rendered unchanged.
  assert.equal(
    prompt.transcript(),
    `${checklist}Enter numbers between 1 and 5, or press Enter to accept.\n${checklist}`,
  );
  assert.deepEqual(prompt.asked(), ["> ", "> ", "Apply this selection? [Y/n] "]);
});

test("declining the confirmation writes nothing at all", async () => {
  const root = vitePlusProject("prompt-decline");
  const prompt = scriptedPrompt(["", "n"]);

  const result = await runInit(root, [], prompt);

  assert.equal(result.plan, null);
  assert.deepEqual(result.written, []);
  assert.deepEqual(result.commands, []);
  assert.equal(exists(root, "vize.config.ts"), false);
  assert.equal(result.output.split("\n").at(-2), "[vize init] cancelled; nothing was written.");
});

test("an input that ends mid-prompt cancels instead of silently doing nothing", async () => {
  const root = vitePlusProject("prompt-eof");
  // No answers at all: the first question sees a closed input.
  const prompt = scriptedPrompt([]);

  const result = await runInit(root, [], prompt);

  assert.deepEqual(prompt.asked(), ["> "]);
  assert.equal(result.plan, null);
  assert.deepEqual(result.written, []);
  assert.deepEqual(result.commands, []);
  assert.equal(exists(root, "vize.config.ts"), false);
  assert.equal(result.output.split("\n").at(-2), "[vize init] cancelled; nothing was written.");
});

test("an unavailable feature is listed without a number and cannot be toggled", async () => {
  const root = temporaryProject("prompt-unavailable");
  writeManifest(root, { name: "fixture", private: true, type: "module" });
  const prompt = scriptedPrompt(["", "y"]);

  const result = await runInit(root, ["--no-install"], prompt);

  assert.equal(
    prompt.transcript(),
    `${CHECKLIST_HEADER}
  1. [x] oxlint plugin (the oxlint binary reads oxlint.config.ts)
     -  vite plugin or nuxt module (no vite.config or nuxt.config found; the other features work without one)
  2. [x] fmt (vize fmt)
     -  typecheck (vize check) (needs a tsconfig.json)
  3. [x] editor extension (.vscode/extensions.json recommendation)

`,
  );
  assert.deepEqual(
    result.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "configured"],
      ["bundler", "skipped"],
      ["fmt", "configured"],
      ["typecheck", "skipped"],
      ["editor", "configured"],
    ],
  );
});
