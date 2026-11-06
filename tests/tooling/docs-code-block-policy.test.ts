import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const contentRoot = path.join(repoRoot, "docs/content");
const plainLanguages = new Set(["text", "txt", "plain", "plaintext"]);

function markdownFiles(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return markdownFiles(target);
    return entry.isFile() && entry.name.endsWith(".md") ? [target] : [];
  });
}

type Fence = {
  character: string;
  length: number;
  language: string;
  line: number;
  body: string[];
};

function codeFences(file: string): Fence[] {
  const fences: Fence[] = [];
  let current: Fence | undefined;
  const lines = fs.readFileSync(file, "utf8").split("\n");

  for (const [index, line] of lines.entries()) {
    const match = /^\s*(`{3,}|~{3,})(.*)$/u.exec(line);
    if (!match) {
      current?.body.push(line);
      continue;
    }

    const marker = match[1];
    if (current) {
      if (marker[0] === current.character && marker.length >= current.length) {
        fences.push(current);
        current = undefined;
      } else {
        current.body.push(line);
      }
      continue;
    }

    current = {
      character: marker[0],
      length: marker.length,
      language: match[2].trim().split(/\s+/u)[0] ?? "",
      line: index + 1,
      body: [],
    };
  }

  assert.equal(current, undefined, `${path.relative(repoRoot, file)} has an unclosed code fence`);
  return fences;
}

void test("documentation code blocks use a semantic highlighter", () => {
  const violations: string[] = [];
  for (const file of markdownFiles(contentRoot)) {
    for (const fence of codeFences(file)) {
      if (!fence.language || plainLanguages.has(fence.language)) {
        violations.push(
          `${path.relative(repoRoot, file)}:${fence.line} uses ${fence.language || "an unlabeled fence"}`,
        );
      }
    }
  }
  assert.deepEqual(violations, []);
});

void test("Vite+ configuration examples compose Vue settings with withVue", () => {
  const violations: string[] = [];
  const scriptLanguages = new Set(["js", "jsx", "ts", "tsx", "javascript", "typescript"]);
  for (const file of markdownFiles(contentRoot)) {
    for (const fence of codeFences(file)) {
      const source = fence.body.join("\n");
      const viteConfig =
        scriptLanguages.has(fence.language) &&
        (/from\s+["'](?:vite|vite-plus|@vizejs\/vite-plugin\/vite-plus)["']/u.test(source) ||
          /\/\/\s*vite\.config\.[cm]?[jt]s/u.test(source));
      if (viteConfig && !source.includes(".withVue(")) {
        violations.push(`${path.relative(repoRoot, file)}:${fence.line}`);
      }
    }
  }
  assert.deepEqual(violations, []);
});

void test("withVue examples export the fluent chain directly", () => {
  const violations: string[] = [];
  for (const file of markdownFiles(contentRoot)) {
    for (const fence of codeFences(file)) {
      const source = fence.body.join("\n");
      if (!source.includes(".withVue(")) continue;
      const storesChain = /\b(?:const|let|var)\s+\w+\s*=\s*defineConfig\s*\(/u.test(source);
      const variableArgument =
        /export\s+default\s+defineConfig\(\s*[A-Za-z_$][\w$]*\s*[,)]/u.test(source) ||
        /\.withVue\(\s*[A-Za-z_$][\w$]*\s*[,)]/u.test(source);
      if (storesChain || variableArgument) {
        violations.push(`${path.relative(repoRoot, file)}:${fence.line}`);
      }
    }
  }
  assert.deepEqual(violations, []);
});
