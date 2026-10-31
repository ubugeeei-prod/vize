import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

type ObjectValue = Record<string, unknown>;
interface Runner {
  lintText(source: string, options: { filePath: string }): Promise<unknown[]>;
}
interface Engine {
  ESLint: new (options: ObjectValue) => Runner;
}
interface Yaml {
  parseAllDocuments(source: string): { errors: unknown[]; toJS(): unknown }[];
}
function record(value: unknown): ObjectValue {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as ObjectValue;
}
const root = fileURLToPath(new URL("../../", import.meta.url));
const rootRequire = createRequire(path.join(root, "package.json"));
const oracle = createRequire(
  fs.realpathSync(path.join(root, "node_modules/@nuxt/eslint-config/package.json")),
);
const importManifest = oracle.resolve("eslint-plugin-import-x/package.json");
const packageRoot = path.dirname(importManifest),
  provider = createRequire(importManifest);
const engine = provider("eslint") as Engine;
const patchHash = "18ff3cf468516ff3335e910419a6118af63fb6db7cadccbadfbd6d7869a0624a";
const expectedSources = [
  ["lib/index.cjs", "65de324b17fb3869fc2fc9627fd3968fb32664fd18a64baab2cf8e637284a57d"],
  [
    "lib/rules/no-internal-modules.js",
    "1d5c8f1dd986dca5e6e3cadcb09027954b94ff51b91a22e4128893cf81dc3f21",
  ],
] as const;
const hash = (bytes: Buffer | string): string => createHash("sha256").update(bytes).digest("hex");
function assertInstalledIdentity(): void {
  assert.equal(
    record(JSON.parse(fs.readFileSync(importManifest, "utf8")) as unknown).version,
    "4.17.1",
  );
  assert.equal(record(provider("minimatch/package.json") as unknown).version, "11.0.1");
  assert.equal(record(provider("eslint/package.json") as unknown).version, "10.13.0");
  assert.equal(record(provider("minimatch") as unknown).makeRe, undefined);
  const constructor = record(provider("minimatch") as unknown).Minimatch;
  assert.equal(typeof constructor, "function");
  assert.ok(!("makeRe" in (constructor as { prototype: object }).prototype));
  for (const [file, expected] of expectedSources)
    assert.equal(hash(fs.readFileSync(path.join(packageRoot, file))), expected);
  assert.equal(
    hash(fs.readFileSync(path.join(root, "patches/eslint-plugin-import-x@4.17.1.patch"))),
    patchHash,
  );
  const yaml = rootRequire("yaml") as Yaml;
  const lock = yaml.parseAllDocuments(fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"));
  assert.equal(lock.length, 2);
  for (const document of lock) assert.deepEqual(document.errors, []);
  const project = record(lock[1]!.toJS());
  assert.equal(record(project.patchedDependencies)["eslint-plugin-import-x@4.17.1"], patchHash);
  const workspace = yaml.parseAllDocuments(
    fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"),
  );
  assert.equal(workspace.length, 1);
  assert.deepEqual(workspace[0]!.errors, []);
  assert.equal(
    record(record(workspace[0]!.toJS()).patchedDependencies)["eslint-plugin-import-x@4.17.1"],
    "patches/eslint-plugin-import-x@4.17.1.patch",
  );
  const snapshots = record(project.snapshots);
  const parents = Object.keys(snapshots).filter((key) =>
    key.startsWith("@nuxt/eslint-config@1.16.0("),
  );
  assert.equal(parents.length, 1);
  const reference = record(record(snapshots[parents[0]!]).dependencies)["eslint-plugin-import-x"];
  assert.equal(typeof reference, "string");
  assert.ok((reference as string).startsWith("4.17.1(patch_hash=" + patchHash + ")"));
  assert.ok("eslint-plugin-import-x@" + reference in snapshots);
}
async function installedPlugins(): Promise<ObjectValue[]> {
  const normalize = (value: unknown): ObjectValue => {
    const module = record(value),
      plugin = module.rules ? module : record(module.default);
    assert.ok(record(plugin.rules)["no-internal-modules"]);
    return plugin;
  };
  return [
    normalize(await import(pathToFileURL(path.join(packageRoot, "lib/index.js")).href)),
    normalize(provider(path.join(packageRoot, "lib/index.cjs")) as unknown),
  ];
}
function makeProject(): string {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-import-x-min11-"));
  for (const directory of ["src/x", "lib", "node_modules/pkg"])
    fs.mkdirSync(path.join(scratch, directory), { recursive: true });
  fs.writeFileSync(path.join(scratch, "package.json"), '{"type":"module","private":true}\n');
  fs.writeFileSync(
    path.join(scratch, "node_modules/pkg/package.json"),
    '{"name":"pkg","version":"1.0.0","type":"module"}\n',
  );
  for (const file of [
    "index.js",
    "src/index.js",
    "lib/public.js",
    "lib/secret.js",
    "lib/hidden.js",
    "node_modules/pkg/internal.js",
  ])
    fs.writeFileSync(path.join(scratch, file), "export default 1;\n");
  return scratch;
}
function lint(
  plugin: ObjectValue,
  scratch: string,
  options: ObjectValue,
  source: string,
): Promise<unknown[]> {
  const runner = new engine.ESLint({
    cwd: scratch,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.js"],
        plugins: { "import-x": plugin },
        languageOptions: { ecmaVersion: 2022, sourceType: "module" },
        rules: { "import-x/no-internal-modules": ["error", options] },
      },
    ],
  });
  return runner.lintText(source, { filePath: path.join(scratch, "src/entry.js") });
}
function cleanResult(filePath: string): ObjectValue {
  return {
    filePath,
    messages: [],
    suppressedMessages: [],
    errorCount: 0,
    fatalErrorCount: 0,
    warningCount: 0,
    fixableErrorCount: 0,
    fixableWarningCount: 0,
    usedDeprecatedRules: [],
  };
}
async function runDefaultHostiles(): Promise<void> {
  assertInstalledIdentity();
  const scratch = makeProject();
  try {
    const filename = "a".repeat(128) + "c.js";
    fs.writeFileSync(path.join(scratch, "lib", filename), "export default 1;\n");
    const source = `import value from "../lib/${filename}";\n`,
      plugins = await installedPlugins();
    const controls = [
      { id: "default-repeated-stars", pattern: "**/" + "*a".repeat(24) + "b.js", errors: 0 },
      { id: "default-nested-alternatives", pattern: "**/+(a|aa)*b.js", errors: 0 },
      { id: "same-path-positive-participation", pattern: "**/*.js", errors: 1 },
    ];
    const outcomes: ObjectValue[] = [];
    for (const control of controls) {
      let previous: unknown[] | undefined;
      for (let build = 0; build < plugins.length; build++) {
        const output = await lint(plugins[build]!, scratch, { forbid: [control.pattern] }, source);
        assert.equal(output.length, 1);
        assert.equal(record(output[0]).errorCount, control.errors);
        const expected = cleanResult(path.join(scratch, "src/entry.js"));
        if (control.errors === 1)
          Object.assign(expected, {
            messages: [
              {
                ruleId: "import-x/no-internal-modules",
                severity: 2,
                message: `Reaching to "../lib/${filename}" is not allowed.`,
                line: 1,
                column: 19,
                messageId: "noAllowed",
                endLine: 1,
                endColumn: 160,
              },
            ],
            errorCount: 1,
            source,
          });
        assert.deepEqual(output, [expected]);
        if (previous) assert.deepEqual(output, previous);
        else previous = output;
        outcomes.push({ control: control.id, build: build === 0 ? "esm" : "cjs", output });
      }
    }
    process.stdout.write(
      JSON.stringify({
        node: process.versions.node,
        defaultConstructorOptions: true,
        oldEngineExecuted: false,
        outcomes,
      }) + "\n",
    );
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
}

if (process.argv.includes("--import-x-default-hostile-child")) {
  await runDefaultHostiles();
} else {
  test("the frozen workspace ImportX caller has the exact secure patch and provider", () => {
    assertInstalledIdentity();
    console.log(
      JSON.stringify({
        node: process.versions.node,
        packageRoot,
        minimatch: provider.resolve("minimatch"),
        patchHash,
        lockSha256: hash(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
      }),
    );
  });
  test("both installed callers retain complete historical results and explicit secure parent semantics", async () => {
    const baseline = record(
      JSON.parse(
        fs.readFileSync(
          new URL("./fixtures/import-x-minimatch-baseline.json", import.meta.url),
          "utf8",
        ),
      ) as unknown,
    );
    assert.ok(Array.isArray(baseline.rows));
    const rows = baseline.rows.map((value) => {
      assert.equal(typeof value, "string");
      return record(JSON.parse(value as string) as unknown);
    });
    assert.equal(rows.length, 64);
    const capturePath =
      "/private/tmp/vize-overnight-20261031/p0/minimatch-compat/eslint-project/src/entry.js";
    const restoreHistory = (row: ObjectValue): ObjectValue => {
      assert.ok(Array.isArray(row.results));
      return {
        ...row,
        results: row.results.map((value) => {
          const result = record(value);
          assert.equal(result.filePath, "<fixture>/src/entry.js");
          return { ...result, filePath: capturePath };
        }),
      };
    };
    const original = rows.map(restoreHistory);
    assert.equal(
      hash(JSON.stringify(original, null, 2) + "\n"),
      "88d820da1dd1701c953b19d0f4c00cdb2cb7a8f316a1db812c1bdfc6e30383e4",
    );
    assert.equal(
      hash(JSON.stringify(original.slice(0, 48), null, 2) + "\n"),
      "ea9370668d6e283e72c1e0aa05299f18df60327b5db5345a2dad8a5fcb521ec5",
    );
    for (let offset = 0; offset < 48; offset += 4)
      for (let build = 1; build < 4; build++)
        assert.deepEqual(rows[offset + build]!.results, rows[offset]!.results);
    const scratch = makeProject();
    try {
      const plugins = await installedPlugins(),
        filePath = path.join(scratch, "src/entry.js");
      const secure = record(provider("minimatch") as unknown);
      assert.equal(typeof secure.minimatch, "function");
      for (const value of ["", "/", scratch, path.join(scratch, "src")])
        assert.equal(
          Reflect.apply(
            secure.minimatch as (value: string, pattern: string) => boolean,
            undefined,
            [value, "?(a)"],
          ),
          false,
        );
      for (const row of rows) {
        assert.equal(typeof row.variant, "string");
        assert.equal(typeof row.source, "string");
        assert.ok(Array.isArray(row.results));
        const expected = row.results.map((value) => ({ ...record(value), filePath }));
        const intentional = row.fixture === "normalized-directory-optional-pattern";
        if (intentional)
          assert.equal(
            record(row.results[0]).errorCount,
            (row.variant as string).startsWith("baseline") ? 3 : 0,
          );
        const plugin = plugins[(row.variant as string).endsWith("esm") ? 0 : 1]!;
        const actual = await lint(plugin, scratch, record(row.options), row.source as string);
        assert.deepEqual(
          actual,
          intentional ? [cleanResult(filePath)] : expected,
          String(row.fixture) + "/" + row.variant,
        );
      }
      // 64 captured row bindings use 32 distinct fixture/build combinations.
      console.log(
        "ImportX: 64 whole target result comparisons; original 48 historical rows authenticated; optional-parent old3/target0 explicit; no old matcher executed",
      );
    } finally {
      fs.rmSync(scratch, { recursive: true, force: true });
    }
  });
  test("actual default-option callers reject bounded hostile nonmatches and participate on the same path", () => {
    const child = spawnSync(
      process.execPath,
      [fileURLToPath(import.meta.url), "--import-x-default-hostile-child"],
      { encoding: "utf8", timeout: 10_000, maxBuffer: 1024 * 1024 },
    );
    assert.equal(child.error, undefined);
    assert.equal(child.signal, null);
    assert.equal(child.status, 0, child.stderr);
    assert.equal(child.stderr, "");
    const receipt = record(JSON.parse(child.stdout) as unknown);
    assert.equal(receipt.node, process.versions.node);
    assert.equal(receipt.defaultConstructorOptions, true);
    assert.equal(receipt.oldEngineExecuted, false);
    assert.ok(Array.isArray(receipt.outcomes));
    assert.equal(receipt.outcomes.length, 6);
    console.log(
      JSON.stringify({
        defaultCallerHostileStdoutSha256: hash(child.stdout),
        bytes: Buffer.byteLength(child.stdout),
        childNode: receipt.node,
        wholeOutputs: receipt.outcomes.length,
        deadlineMilliseconds: 10_000,
        receipt,
      }),
    );
  });
}
