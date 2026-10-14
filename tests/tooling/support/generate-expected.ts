#!/usr/bin/env node
/**
 * Generate expected outputs from Vue's official compiler.
 *
 * Usage:
 *   node tests/tooling/support/generate-expected.ts              # All modes
 *   node tests/tooling/support/generate-expected.ts sfc/basic    # Specific fixture
 *   node tests/tooling/support/generate-expected.ts --mode vdom  # All vdom fixtures
 *   node tests/tooling/support/generate-expected.ts --mode vapor # All vapor fixtures
 *   node tests/tooling/support/generate-expected.ts --mode sfc   # All sfc fixtures
 *
 * Fixtures are Pkl modules (`*.pkl`, evaluated with the `pkl` CLI the fixture
 * runner also uses); a legacy `*.toml` fixture is read only when no Pkl
 * sibling exists.
 *
 * Vapor outputs come from the catalog `@vue/compiler-vapor`, which the gated
 * fixture coverage compares against. `--vapor-supported` uses the newest
 * supported release (`vue-vapor-runtime-supported`) instead, to measure what
 * a newer runtime protocol changes; do not commit that output while coverage
 * still requires every Vapor case to match:
 *
 *   node tests/tooling/support/generate-expected.ts --mode vapor --vapor-supported
 */

import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import * as fs from "node:fs";
import * as path from "node:path";
import { parse as parseTOML } from "@iarna/toml";
import { compileScript, compileTemplate, parse as parseSFC } from "@vue/compiler-sfc";

interface TestCase {
  name: string;
  input: string;
  options?: Record<string, any>;
}

interface TestFixture {
  mode?: string;
  cases: TestCase[];
}

const fixturesDir = path.join(import.meta.dirname!, "..", "..", "fixtures");
const expectedDir = path.join(import.meta.dirname!, "..", "..", "expected");

function pklCommand(): string {
  for (let dir = import.meta.dirname!; ; dir = path.dirname(dir)) {
    for (const candidate of ["node_modules/.bin/pkl", "node_modules/@pkl-community/pkl/pkl"]) {
      const resolved = path.join(dir, candidate);
      if (fs.existsSync(resolved)) return resolved;
    }
    if (path.dirname(dir) === dir) return process.env.PKL ?? "pkl";
  }
}

function loadFixture(filePath: string): TestFixture {
  if (filePath.endsWith(".pkl")) {
    const json = execFileSync(pklCommand(), ["eval", "--format", "json", filePath], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    });
    return JSON.parse(json) as TestFixture;
  }
  const content = fs.readFileSync(filePath, "utf-8");
  return parseTOML(content) as any;
}

/** The fixture file for `base` (a path without extension): Pkl first. */
function fixtureFile(base: string): string | undefined {
  return [".pkl", ".toml"].map((ext) => base + ext).find((file) => fs.existsSync(file));
}

function compileVdom(source: string, options?: Record<string, any>): string {
  const result = compileTemplate({
    source,
    filename: "test.vue",
    id: "test",
    compilerOptions: {
      mode: "module",
      prefixIdentifiers: true,
      hoistStatic: options?.hoistStatic ?? false,
      cacheHandlers: options?.cacheHandlers ?? false,
      ssr: options?.ssr ?? false,
    },
  });
  if (result.errors.length > 0) {
    return `Compile errors: ${result.errors.map((e) => (typeof e === "string" ? e : e.message)).join(", ")}`;
  }
  return result.code;
}

let _compileVaporFn: any = null;
async function loadVaporCompiler() {
  if (!_compileVaporFn) {
    if (process.argv.includes("--vapor-supported")) {
      const release = await import("./vue-vapor-release.mjs");
      _compileVaporFn = release.supportedCompilerVapor.compile;
      console.log(`Vapor expectations: @vue/compiler-vapor ${release.vueVaporSupportedVersion}`);
    } else {
      // The tests package depends on `@vue/compiler-sfc` only; its own
      // `@vue/compiler-vapor` is the matching catalog release.
      const fromSfc = createRequire(createRequire(import.meta.url).resolve("@vue/compiler-sfc"));
      _compileVaporFn = fromSfc("@vue/compiler-vapor").compile;
    }
  }
  return _compileVaporFn;
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function compileVapor(source: string, _options?: Record<string, any>): string {
  if (!_compileVaporFn) {
    return `// @vue/compiler-vapor not loaded`;
  }
  try {
    const result = _compileVaporFn(source, {
      mode: "module",
      prefixIdentifiers: false,
    });
    return result.code;
  } catch (e) {
    return `// Compile error: ${formatUnknownError(e)}`;
  }
}

function compileSFC(source: string, filename: string = "test.vue"): string {
  const { descriptor, errors } = parseSFC(source, { filename });

  if (errors.length > 0) {
    return `Parse errors: ${errors.map((e) => e.message).join(", ")}`;
  }

  let code = "";

  // Compile template if present
  let templateResult: { code: string } | null = null;
  if (descriptor.template) {
    const bindings = descriptor.scriptSetup
      ? compileScript(descriptor, {
          id: filename,
          templateOptions: {},
        }).bindings
      : undefined;

    templateResult = compileTemplate({
      source: descriptor.template.content,
      filename,
      id: filename,
      compilerOptions: {
        bindingMetadata: bindings,
        mode: "module",
      },
    });
  }

  // Compile script setup
  if (descriptor.scriptSetup) {
    const result = compileScript(descriptor, {
      id: filename,
      inlineTemplate: true,
      templateOptions: templateResult
        ? {
            compilerOptions: {
              mode: "module",
            },
          }
        : undefined,
    });
    code = result.content;
  } else if (descriptor.script) {
    // Options API - rewrite export default
    const scriptContent = descriptor.script.content;
    const rewritten = scriptContent.replace(/export\s+default\s+\{/, "const _sfc_main = {");

    if (templateResult) {
      // The render function is attached as `_sfc_main.render` instead of being
      // exported, so rename `export function render` to `function _sfc_render`.
      const renderCode = templateResult.code.replace(
        "export function render(",
        "function _sfc_render(",
      );
      // Match @vue/compiler-sfc / plugin-vue ordering: the <script> block (and
      // its imports) comes first, then the template render block.
      code =
        rewritten +
        "\n" +
        renderCode +
        "\n_sfc_main.render = _sfc_render\nexport default _sfc_main";
    } else {
      code = rewritten + "\nexport default _sfc_main";
    }
  } else if (templateResult) {
    // Template only
    code = templateResult.code;
  }

  return code;
}

function formatSnapFile(
  cases: Array<{ name: string; input: string; output: string; mode: string; css?: string }>,
): string {
  let content = "";

  for (const testCase of cases) {
    content += `===\n`;
    content += `name: ${testCase.name}\n`;
    content += `options: ${testCase.mode}\n`;
    content += `--- INPUT ---\n`;
    content += testCase.input.trim() + "\n";
    content += `--- OUTPUT ---\n`;
    content += testCase.output.trim() + "\n";
    if (testCase.css) {
      content += `--- CSS ---\n`;
      content += testCase.css.trim() + "\n";
    }
  }

  return content;
}

function processFixture(fixturePath: string, outputPath: string) {
  console.log(`Processing: ${path.relative(fixturesDir, fixturePath)}`);

  const fixture = loadFixture(fixturePath);
  const mode = fixture.mode || "vdom";
  const results: Array<{ name: string; input: string; output: string; mode: string }> = [];

  for (const testCase of fixture.cases) {
    try {
      let output: string;
      switch (mode) {
        case "vdom":
          output = compileVdom(testCase.input, testCase.options);
          break;
        case "vapor":
          output = compileVapor(testCase.input, testCase.options);
          break;
        case "sfc":
          output = compileSFC(testCase.input, "test.vue");
          break;
        default:
          output = `Unknown mode: ${mode}`;
      }
      results.push({
        name: testCase.name,
        input: testCase.input,
        output,
        mode,
      });
    } catch (error) {
      console.error(`  Error compiling "${testCase.name}":`, error);
      results.push({
        name: testCase.name,
        input: testCase.input,
        output: `Compile error: ${formatUnknownError(error)}`,
        mode,
      });
    }
  }

  // Ensure output directory exists
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });

  // Write snap file
  const content = formatSnapFile(results);
  fs.writeFileSync(outputPath, content);
  console.log(`  Written: ${path.relative(expectedDir, outputPath)} (${results.length} cases)`);
}

function processDir(dirName: string) {
  const dir = path.join(fixturesDir, dirName);
  if (!fs.existsSync(dir)) return;
  const names = new Set(
    fs
      .readdirSync(dir)
      .filter((f) => /\.(pkl|toml)$/u.test(f) && f !== "CompilerFixture.pkl")
      .map((f) => f.replace(/\.(pkl|toml)$/u, "")),
  );
  for (const name of [...names].sort()) {
    const fixturePath = fixtureFile(path.join(dir, name))!;
    const outputPath = path.join(expectedDir, dirName, `${name}.snap`);
    processFixture(fixturePath, outputPath);
  }
}

async function main() {
  // Pre-load vapor compiler
  await loadVaporCompiler().catch(() => {
    console.warn("Warning: @vue/compiler-vapor not available, vapor fixtures will be skipped");
  });
  const args = process.argv.slice(2);

  // Handle --mode flag
  const modeIdx = args.indexOf("--mode");
  if (modeIdx !== -1 && args[modeIdx + 1]) {
    const mode = args[modeIdx + 1];
    processDir(mode);
    return;
  }

  // If specific fixture is provided
  if (args.length > 0 && !args[0].startsWith("--")) {
    for (const arg of args) {
      const fixturePath = fixtureFile(path.join(fixturesDir, arg));
      const outputPath = path.join(expectedDir, `${arg}.snap`);

      if (!fixturePath) {
        console.error(`Fixture not found: ${path.join(fixturesDir, arg)}.pkl`);
        continue;
      }

      processFixture(fixturePath, outputPath);
    }
    return;
  }

  // Process all modes
  for (const mode of ["vdom", "vapor", "sfc"]) {
    processDir(mode);
  }
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
