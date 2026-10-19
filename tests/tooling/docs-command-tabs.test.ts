import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { reviewedSetupCommand } from "../../docs/scripts/command-tab-render-assertions.ts";
import "../../docs/theme/command-variants.js";

type Choice = { manager: string; command: string | null; gap: string };
const { choices } = (
  globalThis as unknown as {
    __vizeDocsCommands: { choices: (source: string) => Choice[] | null };
  }
).__vizeDocsCommands;

void test("installation choices include every requested manager in order", () => {
  assert.deepEqual(choices("vp install -D @vizejs/vite-plugin\n"), [
    { manager: "vp", command: "vp install -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "npm", command: "npm install -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "pnpm", command: "pnpm add -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "yarn", command: "yarn add -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "bun", command: "bun add -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "aube", command: "aube add -D @vizejs/vite-plugin\n", gap: "workspace" },
    { manager: "jsr", command: null, gap: "registry" },
  ]);
});

void test("bare installs preserve the dependency installation operation", () => {
  assert.deepEqual(choices("vp install\n"), [
    { manager: "vp", command: "vp install\n", gap: "workspace" },
    { manager: "npm", command: "npm install\n", gap: "workspace" },
    { manager: "pnpm", command: "pnpm install\n", gap: "workspace" },
    { manager: "yarn", command: "yarn install\n", gap: "workspace" },
    { manager: "bun", command: "bun install\n", gap: "workspace" },
    { manager: "aube", command: "aube install\n", gap: "workspace" },
    { manager: "jsr", command: null, gap: "registry" },
  ]);
});

void test("Vite+ tasks keep their runner when using another package manager", () => {
  const result = choices("vp dev\nvp build\nvp run check -- --fix\n")!;
  assert.equal(
    result[1].command,
    "npm exec -- vp dev\nnpm exec -- vp build\nnpm exec -- vp run check -- --fix\n",
  );
  assert.equal(
    result[5].command,
    "aube exec vp dev\naube exec vp build\naube exec vp run check -- --fix\n",
  );
});

void test("installed binaries, one-off tools, and project scripts stay distinct", () => {
  const source = "vp exec vize check\nvpx vize init --dry-run\nnpm run typecheck\n";
  assert.equal(
    choices(source)![2].command,
    "pnpm exec vize check\npnpm dlx vize init --dry-run\npnpm run typecheck\n",
  );
  assert.equal(
    choices(source)![4].command,
    "bun x --no-install vize check\nbun x vize init --dry-run\nbun run typecheck\n",
  );
});

void test("support gaps retain original shell examples instead of guessing flags", () => {
  for (const source of [
    "vp install --frozen-lockfile",
    "vp install -D",
    "vp install -E --save-dev",
    "npm install --save-exact",
    "pnpm add -D",
    "vp add",
    "vp run --filter './npm/native' build && echo done",
    "npm install -g vize",
  ]) {
    assert.deepEqual(
      choices(source)!.map((value) => value.command),
      [null, null, null, null, null, null, null],
    );
  }
  assert.equal(choices("cargo install vize\n"), null);
  assert.equal(choices("# npm install vize\n"), null);
});

void test("localized setup guides receive the same operation choices without editing source", () => {
  const root = new URL("../../docs/content/", import.meta.url);
  for (const locale of ["", "ja/", "zh-CN/", "pt-BR/", "fr/"]) {
    const text = readFileSync(new URL(`${locale}getting-started.md`, root), "utf8");
    const commands = [...text.matchAll(/^```(?:bash|sh)\n([^]*?)^```/gm)].map((match) => match[1]);
    assert.ok(commands.length > 0, locale);
    assert.ok(
      commands.some((source) => choices(source)?.[1].command !== null),
      locale,
    );
    for (const source of commands) {
      const result = choices(source);
      if (result)
        assert.deepEqual(
          result.map((choice) => choice.manager),
          ["vp", "npm", "pnpm", "yarn", "bun", "aube", "jsr"],
        );
    }
  }
});

void test("every guide/integration shell example converts to all managers in order", () => {
  const root = new URL("../../docs/content/", import.meta.url).pathname;
  const files: string[] = [];
  const visit = (directory: string) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = join(directory, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.name.endsWith(".md")) files.push(file);
    }
  };
  visit(root);
  let eligible = 0;
  for (const file of files) {
    if (!/\/(?:guide|integrations)\//.test(file)) continue;
    const text = readFileSync(file, "utf8");
    for (const match of text.matchAll(/^```(?:bash|sh|shell)\n([^]*?)^```/gm)) {
      const result = choices(match[1]);
      if (!result) continue;
      eligible++;
      assert.deepEqual(
        result.map((choice) => choice.manager),
        ["vp", "npm", "pnpm", "yarn", "bun", "aube", "jsr"],
        file,
      );
    }
  }
  assert.ok(eligible > 0, "guide shell examples were converted");
});

void test("reviewed setup routes start with the command the tab renderer verifies", () => {
  const root = new URL("../../docs/content/", import.meta.url);
  const verified = { install: "vp install -D @vizejs/vite-plugin", init: "vpx vize init" };
  const cases = [
    "/getting-started",
    "/ja/getting-started",
    "/fr/getting-started",
    "/pt-BR/getting-started",
    "/zh-CN/getting-started",
    "/guide/init",
  ];
  for (const route of cases) {
    const policy = reviewedSetupCommand(route);
    assert(policy, `${route}: reviewed setup policy`);
    const source = readFileSync(new URL(`${route.slice(1)}.md`, root), "utf8");
    // The first tab widget mounts on the first convertible shell fence.
    const first = [...source.matchAll(/^```(?:bash|sh|shell)\n([^]*?)^```/gm)].find((match) =>
      choices(match[1]),
    );
    assert(first, `${route}: a tabbed shell recipe exists`);
    assert.equal(first[1].trim(), verified[policy], `${route}: first tabbed command`);
  }
  for (const route of ["/guide/configuration", "/fr/guide/init", "/other/getting-started"])
    assert.equal(reviewedSetupCommand(route), undefined, `${route}: no unreviewed policy`);
});
