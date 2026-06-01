import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { npmxApp } from "../../_helpers/apps.ts";

const app = npmxApp;
const VITE_PLUS_BIN = `${process.env.HOME ?? ""}/.vite-plus/bin`;

describe(`${app.name} build`, () => {
  before(() => {
    if (!process.env.RUN_BUILD_TESTS) {
      console.log("Skipping: build tests are opt-in (RUN_BUILD_TESTS=1)");
      process.exit(0);
    }
    if (app.setup) app.setup();
  });

  it("build succeeds", () => {
    const build = app.build!;
    const cmd = `${build.command} ${build.args.join(" ")}`;
    console.log(`Running: ${cmd} (cwd: ${app.cwd})`);

    execSync(cmd, {
      cwd: app.cwd,
      env: {
        ...process.env,
        PATH: `${VITE_PLUS_BIN}:${process.env.PATH}`,
        NODE_ENV: "production",
        ...app.env,
      },
      stdio: "inherit",
      timeout: build.timeout,
    });

    console.log("Build completed successfully");

    const publicDir = path.join(app.cwd, ".output", "public");
    const references = collectVizeComponentsCssReferences(path.join(app.cwd, ".output"));
    for (const reference of references) {
      const componentsCssPath = findPublicAssetPath(publicDir, reference);
      assert.ok(
        componentsCssPath,
        `Nuxt SSR build references ${reference}, so it should exist in public assets`,
      );
      assert.ok(
        fs.statSync(componentsCssPath).size > 0,
        `Nuxt SSR emitted component stylesheet ${reference} should not be empty`,
      );
    }
  });
});

function collectVizeComponentsCssReferences(outputDir: string): string[] {
  const references = new Set<string>();
  visitFiles(outputDir, (filePath) => {
    if (!/\.(?:js|mjs|json)$/.test(filePath)) {
      return;
    }

    const source = fs.readFileSync(filePath, "utf-8");
    for (const match of source.matchAll(/["']([^"']*vize-components\.css)["']/g)) {
      const reference = normalizeVizeComponentsCssReference(match[1]);
      if (reference) {
        references.add(reference);
      }
    }
  });
  return [...references].sort();
}

function normalizeVizeComponentsCssReference(reference: string): string | null {
  if (reference.startsWith("../public/")) {
    return reference.slice("../public/".length);
  }
  if (reference.startsWith("../")) {
    return null;
  }

  return reference.replace(/^\.?\//, "");
}

function findPublicAssetPath(publicDir: string, reference: string): string | null {
  const candidates = [path.join(publicDir, reference)];
  if (!reference.includes("/")) {
    candidates.push(path.join(publicDir, "_nuxt", reference));
  }

  return candidates.find((candidate) => fs.existsSync(candidate)) ?? null;
}

function visitFiles(dir: string, visitor: (filePath: string) => void): void {
  if (!fs.existsSync(dir)) {
    return;
  }

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      visitFiles(entryPath, visitor);
      continue;
    }

    if (entry.isFile()) {
      visitor(entryPath);
    }
  }
}
