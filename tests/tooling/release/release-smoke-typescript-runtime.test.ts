import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertInstalledMapperContract,
  runInstalledContentMapperChecks,
  runRuntimeChecks,
  writeRuntimeSmokeProject,
} from "../../../tools/support/release/smoke_release_install/runtime.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const contract = {
  dynamicConfig: true,
  exec: ["node", "./bin/vize", "content-mapper"],
  compilerOptions: ["checkJs", "noUnusedLocals", "rootDir"],
};
const diagnostics = [
  ["errors/Broken.vue", "TS2322"],
  ["errors/JavaScriptConsumer.js", "TS2322"],
  ["errors/JsxConsumer.jsx", "TS2322"],
  ["src/Unused.vue", "TS6133"],
];
const consumerArgs = [
  "--ignoreConfig",
  "--noEmit",
  "--strict",
  "--module",
  "preserve",
  "--moduleResolution",
  "bundler",
  "--allowArbitraryExtensions",
  "--pretty",
  "false",
  "dist/verify.ts",
];
type Call = { tool: string; args: string[]; cwd: string };

function fixture(run: (f: ReturnType<typeof setup>) => void): void {
  const f = setup();
  const changes = {
    NODE_BIN: process.execPath,
    PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    VIZE_REPO_ROOT: root,
    VIZE_TEST_CONTENT_MAPPER_TSGO: f.tsgo,
    VIZE_SMOKE_RUNTIME_TEST_LOG: f.log,
    VIZE_SMOKE_RUNTIME_TEST_FAULT: "",
  };
  const saved = Object.fromEntries(Object.keys(changes).map((key) => [key, process.env[key]]));
  Object.assign(process.env, changes);
  try {
    run(f);
  } finally {
    for (const [key, value] of Object.entries(saved)) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    fs.rmSync(f.temporary, { recursive: true, force: true });
  }
}
function setup() {
  const temporary = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-runtime 新規 ")),
  );
  const install = path.join(temporary, "install"),
    pkg = path.join(install, "node_modules/vize");
  const bin = path.join(pkg, "bin/vize"),
    log = path.join(temporary, "calls.jsonl");
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  const manifest = (typescript: unknown = { contentMapper: contract }) => {
    fs.writeFileSync(
      path.join(pkg, "package.json"),
      JSON.stringify({ name: "vize", version: "0.0.0-control", typescript }),
    );
  };
  manifest();
  fs.writeFileSync(bin, "#!/usr/bin/env node\n", { mode: 0o755 });
  // These executable protocol peers record every real subprocess invocation.
  // They exercise the installed oracle, not public native release acceptance.
  const tsgoSource = path.join(temporary, "tsgo.cjs");
  fs.writeFileSync(tsgoSource, `#!${process.execPath}\n${protocolPeer}`, { mode: 0o755 });
  const tsgo = process.platform === "win32" ? path.join(temporary, "tsgo.cmd") : tsgoSource;
  if (process.platform === "win32")
    fs.writeFileSync(
      tsgo,
      `@echo off\r\n"${process.execPath}" "${tsgoSource}" %*\r\nexit /b %errorlevel%\r\n`,
    );
  const tsc = path.join(install, "node_modules/typescript/bin/tsc");
  fs.mkdirSync(path.dirname(tsc), { recursive: true });
  fs.writeFileSync(
    tsc,
    `const fs = require("node:fs"); fs.appendFileSync(process.env.VIZE_SMOKE_RUNTIME_TEST_LOG, JSON.stringify({ tool: "tsc", args: process.argv.slice(2), cwd: process.cwd() }) + "\\n");`,
  );
  return {
    temporary,
    install,
    pkg,
    bin,
    manifest,
    tsgo,
    log,
    project: path.join(install, "content mapper project with spaces"),
    calls: (): Call[] =>
      fs
        .readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Call),
  };
}
const protocolPeer = String.raw`
const fs = require("node:fs"), path = require("node:path");
const args = process.argv.slice(2), fault = process.env.VIZE_SMOKE_RUNTIME_TEST_FAULT;
fs.appendFileSync(process.env.VIZE_SMOKE_RUNTIME_TEST_LOG, JSON.stringify({tool:"tsgo",args,cwd:process.cwd()}) + "\n");
if (args.includes("tsconfig.error.json")) {
  if (fault === "false-clean") process.exit(0);
  const diagnostics = [["errors/Broken.vue","TS2322"],["errors/JavaScriptConsumer.js","TS2322"],["errors/JsxConsumer.jsx","TS2322"],["src/Unused.vue","TS6133"]];
  diagnostics.forEach(([file,code],index) => {
    if (fault === "drop-" + index) return;
    const text = fault === "split-diagnostic" && index === 0 ? file + "\n" + code : file + " " + code;
    (index % 2 === 0 ? process.stdout : process.stderr).write(text + "\n");
  });
  process.exit(1);
}
if (args.includes("tsconfig.emit.json")) {
  fs.mkdirSync("dist",{recursive:true});
  if (fault !== "missing-app") fs.writeFileSync(path.join("dist",fault === "alternate-declaration" ? "App.d.vue.ts" : "App.vue.d.ts"), "export {}\n");
  if (fault !== "missing-main") fs.writeFileSync("dist/main.d.ts","export {}\n");
}
`;

test("installed mapper accepts the exact production contract", () => {
  fixture(({ install }) => assertInstalledMapperContract(install));
});
for (const [name, value] of [
  ["corrupted exec", { ...contract, exec: ["node", "./bin/missing", "content-mapper"] }],
  ["missing compilerOptions", { exec: contract.exec }],
  ["retired extensions", { ...contract, extensions: { ".vue": ".tsx" } }],
] as const) {
  test(`installed mapper rejects ${name}`, () =>
    fixture((f) => {
      f.manifest({ contentMapper: value });
      assert.throws(
        () => assertInstalledMapperContract(f.install),
        /must expose the production typescript\.contentMapper contract/,
      );
    }));
}
test("installed mapper rejects the retired top-level contract", () =>
  fixture((f) => {
    fs.writeFileSync(
      path.join(f.pkg, "package.json"),
      JSON.stringify({ name: "vize", version: "0.0.0-control", tsContentMapper: contract }),
    );
    assert.throws(
      () => assertInstalledMapperContract(f.install),
      /must expose the production typescript\.contentMapper contract/,
    );
  }));
test("installed mapper rejects a symlinked package", () =>
  fixture((f) => {
    fs.renameSync(f.pkg, `${f.pkg}.saved`);
    fs.symlinkSync(`${f.pkg}.saved`, f.pkg, process.platform === "win32" ? "junction" : "dir");
    assert.throws(() => assertInstalledMapperContract(f.install), /must not be a symlink/);
  }));
test("installed mapper rejects a missing executable", () =>
  fixture((f) => {
    fs.rmSync(f.bin);
    assert.throws(() => assertInstalledMapperContract(f.install), /bin[/\\]vize must exist/);
  }));
test("installed mapper retains the platform executable policy", () =>
  fixture((f) => {
    fs.chmodSync(f.bin, 0o644);
    if (process.platform === "win32")
      assert.doesNotThrow(() => assertInstalledMapperContract(f.install));
    else assert.throws(() => assertInstalledMapperContract(f.install), /must be executable/);
  }));

for (const fault of ["normal", "alternate-declaration"]) {
  test(`installed mapper verifies ${fault} declarations with both complete compiler commands`, (t) =>
    fixture((f) => {
      process.env.VIZE_SMOKE_RUNTIME_TEST_FAULT = fault;
      const output: unknown[][] = [];
      t.mock.method(console, "log", (...args: unknown[]) => output.push(args));
      runInstalledContentMapperChecks(f.install);
      const common = ["--runExternalCode", "--pretty", "false"];
      assert.deepEqual(f.calls(), [
        { tool: "tsgo", args: [...common, "--noEmit", "-p", "tsconfig.json"], cwd: f.project },
        {
          tool: "tsgo",
          args: [...common, "--noEmit", "-p", "tsconfig.error.json"],
          cwd: f.project,
        },
        { tool: "tsgo", args: [...common, "-p", "tsconfig.emit.json"], cwd: f.project },
        { tool: "tsgo", args: consumerArgs, cwd: f.project },
        { tool: "tsc", args: consumerArgs, cwd: f.project },
      ]);
      assert.deepEqual(output, [
        ["runtime: packed vize Content Mapper check and declaration emit"],
      ]);
      assert.deepEqual(
        fs.readFileSync(path.join(f.project, "dist/verify.ts")),
        fs.readFileSync(
          path.join(root, "crates/vize/tests/fixtures/content_mapper_project/consumer/verify.ts"),
        ),
      );
    }));
}
for (const [fault, message] of [
  ["false-clean", /installed mapper error fixture unexpectedly passed/],
  ["split-diagnostic", /errors\/Broken.vue did not report TS2322/],
  ["missing-app", /installed mapper did not emit an App.vue declaration/],
  ["missing-main", /installed mapper did not emit dist\/main.d.ts/],
  ...diagnostics.map(([file, code], index): [string, RegExp] => [
    `drop-${index}`,
    new RegExp(`${file.replaceAll(".", "\\.")} did not report ${code}`),
  ]),
] as const) {
  test(`installed mapper rejects ${fault}`, () =>
    fixture((f) => {
      process.env.VIZE_SMOKE_RUNTIME_TEST_FAULT = fault;
      assert.throws(() => runInstalledContentMapperChecks(f.install), message);
      assert.equal(
        f.calls().some(({ tool }) => tool === "tsc"),
        false,
      );
    }));
}
test("mapper rejection preserves complete original stdout and stderr", () =>
  fixture((f) => {
    process.env.VIZE_SMOKE_RUNTIME_TEST_FAULT = "drop-0";
    assert.throws(
      () => runInstalledContentMapperChecks(f.install),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          error.message,
          "errors/Broken.vue did not report TS2322:\n" +
            "errors/JsxConsumer.jsx TS2322\n\n" +
            "errors/JavaScriptConsumer.js TS2322\nsrc/Unused.vue TS6133\n",
        );
        return true;
      },
    );
  }));
test("disabled mapper checks start no compiler and require no installed package", () =>
  fixture((f) => {
    process.env.VIZE_TEST_CONTENT_MAPPER_TSGO = "";
    fs.rmSync(f.pkg, { recursive: true });
    runInstalledContentMapperChecks(f.install);
    assert.equal(fs.existsSync(f.log), false);
  }));

test("runtime writer preserves every Rust text literal and the complete TypeScript configuration", () =>
  fixture((f) => {
    writeRuntimeSmokeProject(f.install);
    const rust = fs.readFileSync(
      path.join(root, "tools/commands/release/npm/smoke-release-install.rs"),
      "utf8",
    );
    const section = rust.slice(
      rust.indexOf("fn write_runtime_smoke_project"),
      rust.indexOf("fn run_installed_content_mapper_checks"),
    );
    for (const filename of ["index.html", "vite.config.mjs", "App.vue", "main.ts"]) {
      const escaped = filename.replaceAll(".", "\\.");
      const literal = new RegExp(`\\.join\\("${escaped}"\\),\\s*("(?:[^"\\\\]|\\\\.)*")`).exec(
        section,
      )?.[1];
      assert.ok(literal, filename);
      const directory = filename === "App.vue" || filename === "main.ts" ? "src" : "";
      assert.equal(
        fs.readFileSync(path.join(f.install, directory, filename), "utf8"),
        JSON.parse(literal) as string,
      );
    }
    assert.equal(
      fs.readFileSync(path.join(f.install, "tsconfig.json"), "utf8"),
      JSON.stringify(
        {
          compilerOptions: {
            lib: ["ES2022", "DOM", "DOM.Iterable"],
            module: "ESNext",
            moduleResolution: "Bundler",
            strict: true,
            target: "ES2022",
            types: [],
          },
          include: ["src/**/*.ts", "src/**/*.vue"],
        },
        null,
        2,
      ) + "\n",
    );
  }));
for (const broken of ["require", "import"]) {
  test(`installed native oracle refuses the broken ${broken} API`, () =>
    fixture((f) => {
      const native = path.join(f.install, "node_modules/@vizejs/native");
      fs.mkdirSync(native, { recursive: true });
      fs.writeFileSync(
        path.join(native, "package.json"),
        JSON.stringify({
          name: "@vizejs/native",
          exports: { require: "./require.cjs", import: "./import.mjs" },
        }),
      );
      fs.writeFileSync(
        path.join(native, "require.cjs"),
        broken === "require"
          ? "module.exports = {};"
          : 'module.exports = { compileSfc() { return { errors: [], code: "control" }; } };',
      );
      fs.writeFileSync(path.join(native, "import.mjs"), "export default {};");
      const pkg = {
        name: "@vizejs/native",
        version: "0.0.0-control",
        tarball: "unused",
        compatible: true,
      };
      assert.throws(
        () => runRuntimeChecks(f.install, [pkg], [pkg], f.temporary),
        new RegExp(`compileSfc missing from ${broken} smoke`),
      );
    }));
}
