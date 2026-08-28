import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { smokeWasmPackage } from "../../tools/npm/smoke-wasm-package.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);

function copyRepoFile(targetDir: string, relativePath: string): void {
  fs.copyFileSync(path.join(root, relativePath), path.join(targetDir, path.basename(relativePath)));
}

test("wasm package smoke validates exports, init guards, and runtime APIs", async () => {
  const packageDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-smoke-"));

  copyRepoFile(packageDir, "npm/wasm/package.json");
  copyRepoFile(packageDir, "npm/wasm/index.js");
  copyRepoFile(packageDir, "npm/wasm/index.d.ts");
  copyRepoFile(packageDir, "npm/wasm/lint-format.d.ts");
  fs.writeFileSync(path.join(packageDir, "vize_vitrine.d.ts"), "export {};\n");
  fs.writeFileSync(
    path.join(packageDir, "vize_vitrine_bg.wasm"),
    Buffer.from([0x00, 0x61, 0x73, 0x6d]),
  );
  fs.writeFileSync(
    path.join(packageDir, "vize_vitrine.js"),
    `export default async function init(options) {
  if (options == null || !("module_or_path" in options)) {
    throw new Error("expected wasm-bindgen init options");
  }
  if (options.module_or_path.byteLength === 0) {
    throw new Error("invalid wasm bytes");
  }
  if (options.module_or_path instanceof URL) {
    await fetch(options.module_or_path);
  }
}
export class Compiler {
  compile() {}
  compileVapor() {}
  parse() {}
  parseSfc() {}
  compileSfc() {}
  compileCss() {}
  free() {}
}
export function compile(_template, options = {}) {
  if (options.templateSyntax === "strict") throw new Error("strict template syntax");
  return {
    code: options.runtimeGlobalName ? "const runtime = " + options.runtimeGlobalName : "code",
    preamble: options.runtimeModuleName ? 'from "' + options.runtimeModuleName + '"' : "",
    ast: { children: [{ isSelfClosing: options.vueParserQuirks === true }] },
    map: options.sourceMap
      ? {
          version: 3,
          file: options.filename ?? "template.vue",
          sources: [options.filename ?? "template.vue"],
          sourcesContent: [_template],
          names: [],
          mappings: "",
        }
      : undefined,
  };
}
export function compileVapor() {}
export function parseTemplate(_template, options = {}) {
  if (options.templateSyntax === "standard" && options.vueParserQuirks === true) {
    throw new Error("standard template syntax");
  }
  return { children: [{ isSelfClosing: options.vueParserQuirks === true }] };
}
export function parseSfc() {}
export function compileSfc(_source, options = {}) {
  const runtime =
    options.mode === "function"
      ? options.runtimeGlobalName ?? "Vue"
      : options.runtimeModuleName ?? "vue";
  const filename = options.filename ?? "anonymous.vue";
  return {
    template: {
      code: runtime,
      preamble: runtime,
      ast: {},
      map: options.sourceMap
        ? {
            version: 3,
            file: filename,
            sources: [filename],
            sourcesContent: [_source],
            names: [],
            mappings: "",
          }
        : undefined,
      helpers: [],
    },
    script: {
      code:
        runtime +
        "\\n" +
        (options.scriptExt === "preserve" ? "const count: number = 1" : "const count = 1"),
    },
  };
}
export function compileJsx() {}
export function compileCss() {}
export function lintSfc(_source, options = {}) {
  return {
    filename: options.filename ?? "anonymous.vue",
    errorCount: 0,
    warningCount: 1,
    diagnostics: [{
      help: undefined,
      location: {
        start: { line: 2, column: 7, offset: 17 },
        end: { line: 2, column: 9, offset: 19 },
      },
      message: "Multiple consecutive spaces",
      rule: "vue/no-multi-spaces",
      severity: "warning",
    }],
  };
}
export function formatSfc(source) {
  return { code: source.replace("  id", " id"), changed: true };
}
`,
  );

  await smokeWasmPackage(packageDir);
});

test("wasm package smoke fails when wrapper entrypoint is missing", async () => {
  const packageDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-smoke-missing-"));

  copyRepoFile(packageDir, "npm/wasm/package.json");
  fs.writeFileSync(path.join(packageDir, "index.d.ts"), "export {};\n");
  fs.writeFileSync(
    path.join(packageDir, "vize_vitrine.js"),
    "export default async function init() {}\n",
  );
  fs.writeFileSync(path.join(packageDir, "vize_vitrine.d.ts"), "export {};\n");
  fs.writeFileSync(path.join(packageDir, "vize_vitrine_bg.wasm"), "");

  await assert.rejects(() => smokeWasmPackage(packageDir), /index\.js is missing/);
});

test("wasm package declarations type-check a strict consumer", (t) => {
  const consumerDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-types-"));
  const packageDir = path.join(consumerDir, "node_modules", "@vizejs", "wasm");
  fs.mkdirSync(packageDir, { recursive: true });
  copyRepoFile(packageDir, "npm/wasm/package.json");
  copyRepoFile(packageDir, "npm/wasm/index.d.ts");
  copyRepoFile(packageDir, "npm/wasm/lint-format.d.ts");
  t.after(() => fs.rmSync(consumerDir, { force: true, recursive: true }));

  fs.writeFileSync(
    path.join(consumerDir, "consumer.ts"),
    `import init, {
  compile,
  formatSfc,
  lintSfc,
  type BindingMetadata,
  type CompilerOptions,
  type FormatResult,
  type LintResult,
} from "@vizejs/wasm";

void init();

const bindingMetadata: BindingMetadata = {
  bindings: { message: "setup-ref" },
  propsAliases: {},
  isScriptSetup: true,
};
const compilerOptions: CompilerOptions = {
  mode: "module",
  prefixIdentifiers: true,
  hoistStatic: true,
  cacheHandlers: true,
  scopeId: "data-v-smoke",
  ssr: false,
  sourceMap: true,
  filename: "Smoke.vue",
  outputMode: "vdom",
  isTs: true,
  customRenderer: false,
  templateSyntax: "strict",
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  runtimeModuleName: "@acme/vue-runtime",
  runtimeGlobalName: "AcmeVue",
  scriptExt: "preserve",
  bindingMetadata,
  vueParserQuirks: false,
};
const compiled = compile("<div />", compilerOptions);
compiled.map?.sources[0]?.toUpperCase();

const lint: LintResult = lintSfc("<template />", {
  enabledRules: ["vue/no-dupe-keys"],
  filename: "App.vue",
  locale: "ja",
  preset: "essential",
});
const diagnostic = lint.diagnostics[0];
if (diagnostic) {
  const help: string | undefined = diagnostic.help;
  console.log(help, diagnostic.location.start.offset);
}

const formatted: FormatResult = formatSfc("<template />", {
  attributeGroups: [["id", "class"]],
  maxAttributesPerLine: null,
  printWidth: 80,
  trailingComma: "all",
});
console.log(formatted.code, formatted.changed);

// @ts-expect-error severity overrides are not supported by the WASM binding
lintSfc("<template />", { severityOverrides: {} });
// @ts-expect-error filename is not a formatter option
formatSfc("<template />", { filename: "App.vue" });
// @ts-expect-error this reserved shared option is not implemented by WASM
compile("<div />", { experimentalServerScript: true });
// @ts-expect-error scriptExt is a closed compatibility policy
compile("<div />", { scriptExt: "ts" });
`,
  );
  fs.writeFileSync(
    path.join(consumerDir, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        lib: ["ESNext", "DOM"],
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: false,
        strict: true,
      },
      include: ["consumer.ts"],
    }),
  );

  const configuredChecker = process.env.TSGO_PATH;
  const tscEntry = require.resolve("typescript/bin/tsc");
  const result = configuredChecker
    ? spawnSync(configuredChecker, ["-p", path.join(consumerDir, "tsconfig.json")], {
        encoding: "utf8",
      })
    : spawnSync(process.execPath, [tscEntry, "-p", path.join(consumerDir, "tsconfig.json")], {
        encoding: "utf8",
      });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});
