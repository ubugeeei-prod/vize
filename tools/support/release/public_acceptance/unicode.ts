import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { inspect } from "node:util";

const digest = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const version = "3.6.0-rc.10";
const support = [
  "vapor-sfc-runtime.mjs",
  "vapor-unicode-key-scope.ts",
  "davinci-mounted-trace.mjs",
  "davinci-mounted-child.mjs",
  "davinci-runtime-trace.mjs",
  "davinci-mounted-scope.mjs",
  "mounted-runtime-devtools.ts",
  "mounted-runtime-source.ts",
  "mounted-runtime-native-boundary.ts",
].map((name) => `tests/tooling/support/${name}`);
export const unicodeCases = [
  {
    name: "ascii",
    template: `<p v-for="kind in ['x', 'y']" :key="kind">{{ kind }}</p>`,
    texts: ["x", "y"],
    args: ["x", 0],
    value: "x",
  },
  {
    name: "latin",
    template: `<p v-for="café in ['x']" :key="café">{{ café }}</p>`,
    texts: ["x"],
    args: ["x", 0],
    value: "x",
  },
  {
    name: "unicode",
    template: `<p v-for="種類 in ['x', 'y']" :key="種類">{{ 種類 }}</p>`,
    texts: ["x", "y"],
    args: ["x", 0],
    value: "x",
  },
  {
    name: "member",
    template: `<p v-for="行 in items" :key="行.id">{{ 行.id }}</p>`,
    texts: ["1"],
    args: [{ id: 1 }, 0],
    value: 1,
  },
  {
    name: "index",
    template: `<p v-for="(r, 番号) in items" :key="番号">{{ r.id }}</p>`,
    texts: ["1"],
    args: [{ id: 1 }, 0],
    value: 0,
  },
];
const cases = unicodeCases;
const source = (template: string) =>
  `<script setup vapor>\nconst items = [{ id: 1 }]\n</script>\n<template>${template}</template>\n`;
const options = (production: boolean) => ({
  filename: "App.vue",
  mode: "module",
  vapor: true,
  scopeId: "probe",
  isProd: production,
  inlineTemplate: production,
});
type SourceEntry = { file: string; sha256: string };
type Source = { head: string; files: SourceEntry[] };
type Frame = {
  name: string;
  production: boolean;
  official: boolean;
  stem: string;
  inputSha256: string;
  stdoutSha256: string;
  stderrSha256: string;
  pid: number;
  status: number | null;
  signal: string | null;
  processError: ReturnType<typeof unicodeProcessFailure>;
};
type Receipt = {
  schema: "vize-public-unicode-v1";
  issue: 9493;
  version: string;
  head: string;
  source: Source;
  providers: ReturnType<typeof providers>;
  frames: Frame[];
  success: true;
};
type Native = {
  compileSfc(
    source: string,
    compileOptions: ReturnType<typeof options>,
  ): { code: string; errors: string[]; warnings: string[] };
};

/** Preserve the original spawn failure without JSON silently dropping Error fields. */
export function unicodeProcessFailure(error: unknown) {
  if (error === undefined) return null;
  const settings = {
    depth: null,
    maxArrayLength: null,
    maxStringLength: null,
    showHidden: true,
    customInspect: false,
    getters: false,
  } as const;
  return {
    type: typeof error,
    name: error instanceof Error ? error.name : null,
    message: error instanceof Error ? error.message : null,
    stack: error instanceof Error ? error.stack : null,
    ownProperties:
      error !== null && (typeof error === "object" || typeof error === "function")
        ? inspect(Object.getOwnPropertyDescriptors(error), settings)
        : null,
    raw: inspect(error, settings),
  };
}

/** Keep the existing observer's relative package ownership and exact frozen H bytes. */
export function unicodeSourceIdentity(read: (file: string) => Buffer, head: string): Source {
  return {
    head,
    files: [...support, "tests/package.json"].map((file) => ({ file, sha256: digest(read(file)) })),
  };
}

export function prepareUnicodeSource(
  read: (file: string) => Buffer,
  consumer: string,
  head: string,
): Source {
  const identity = unicodeSourceIdentity(read, head);
  for (const { file, sha256 } of identity.files) {
    const bytes = read(file),
      target = path.join(consumer, file);
    assert.equal(digest(bytes), sha256);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
  }
  return identity;
}

function assertSource(expected: Source, consumer: string) {
  assert.deepEqual(
    expected.files.map((entry) => entry.file),
    [...support, "tests/package.json"],
  );
  for (const entry of expected.files)
    assert.equal(digest(readFileSync(path.join(consumer, entry.file))), entry.sha256, entry.file);
}

function providers(consumer: string) {
  const require = createRequire(path.join(consumer, "tests/package.json"));
  const vuePath = realpathSync(require.resolve("vue-ssr-css-vars-oracle/package.json"));
  const fromVue = createRequire(vuePath);
  const compiler = realpathSync(fromVue.resolve("@vue/compiler-sfc/package.json"));
  const fromCompiler = createRequire(compiler);
  const files = [
    vuePath,
    compiler,
    realpathSync(fromCompiler.resolve("@vue/compiler-vapor/package.json")),
    realpathSync(fromVue.resolve("./dist/vue.runtime-with-vapor.esm-browser.js")),
    realpathSync(fromVue.resolve("./dist/vue.runtime-with-vapor.esm-browser.prod.js")),
    realpathSync(require.resolve("@babel/parser/package.json")),
    realpathSync(require.resolve("@babel/core/package.json")),
  ];
  const lock = readFileSync(path.join(consumer, "package-lock.json"));
  const manifest = JSON.parse(readFileSync(path.join(consumer, "package.json"), "utf8"));
  assert.equal(manifest.dependencies["vue-ssr-css-vars-oracle"], `npm:vue@${version}`);
  for (const name of ["@babel/core", "@babel/parser"])
    assert.equal(manifest.dependencies[name], "7.29.0");
  const installed = JSON.parse(lock.toString()).packages;
  const packages = [vuePath, compiler, files[2], files[5], files[6]].map((file) => {
    const location = path.relative(consumer, path.dirname(file)).split(path.sep).join("/");
    const entry = installed[location];
    const actual = JSON.parse(readFileSync(file, "utf8"));
    assert.equal(entry.version, actual.version);
    assert.equal(entry.link, undefined);
    assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
    assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
    return {
      location,
      name: actual.name,
      version: entry.version,
      resolved: entry.resolved,
      integrity: entry.integrity,
    };
  });
  return {
    lockSha256: digest(lock),
    packages,
    files: files.map((file, index) => {
      assert.ok(file.startsWith(realpathSync(consumer) + path.sep + "node_modules" + path.sep));
      const bytes = readFileSync(file);
      if (index < 3) assert.equal(JSON.parse(bytes.toString()).version, version);
      if (index >= 5) assert.equal(JSON.parse(bytes.toString()).version, "7.29.0");
      return { file, sha256: digest(bytes) };
    }),
  };
}

function frameInput(
  entry: (typeof cases)[number],
  production: boolean,
  official: boolean,
  code: string,
) {
  return {
    backend: "vapor",
    code,
    child: "export default {};",
    production,
    unicodeKeyScope: {
      source: source(entry.template),
      official,
      samples: [{ args: entry.args, value: entry.value }],
    },
  };
}

/** The public NAPI is supplied exclusively by the existing authenticated loader. */
export function observePublishedUnicode(native: Native, publishedVersion: string, output: string) {
  const consumer = process.cwd();
  const preparation = JSON.parse(
    readFileSync(path.join(path.dirname(output), "preparation.json"), "utf8"),
  );
  const expected: Source = preparation.unicode;
  assert.equal(expected.head, preparation.source.head);
  assertSource(expected, consumer);
  const before = providers(consumer),
    directory = path.join(path.dirname(output), "native-unicode");
  mkdirSync(directory);
  const frames: Frame[] = [];
  for (const production of [false, true])
    for (const entry of cases) {
      const authored = source(entry.template),
        compileOptions = options(production);
      const compiled = native.compileSfc(authored, compileOptions);
      const compiledPacket = { source: authored, options: compileOptions, result: compiled };
      writeFileSync(
        path.join(directory, `${entry.name}-${production}.compile.json`),
        JSON.stringify(compiledPacket) + "\n",
        { flag: "wx" },
      );
      assert.deepEqual(compiled.errors, []);
      assert.deepEqual(compiled.warnings, []);
      assert.ok(compiled.code.length > 0);
      for (const official of [false, true]) {
        const input = Buffer.from(
          JSON.stringify(frameInput(entry, production, official, compiled.code)),
        );
        const result = spawnSync(
          process.execPath,
          [path.join(consumer, "tests/tooling/support/vapor-sfc-runtime.mjs")],
          { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024, env: process.env },
        );
        const stem = `${entry.name}-${production}-${official}`,
          stdout = result.stdout ?? Buffer.alloc(0),
          stderr = result.stderr ?? Buffer.alloc(0);
        for (const [suffix, bytes] of [
          ["input", input],
          ["stdout", stdout],
          ["stderr", stderr],
        ] as const)
          writeFileSync(path.join(directory, `${stem}.${suffix}.bin`), bytes, { flag: "wx" });
        const frame: Frame = {
          name: entry.name,
          production,
          official,
          stem,
          inputSha256: digest(input),
          stdoutSha256: digest(stdout),
          stderrSha256: digest(stderr),
          pid: result.pid,
          status: result.status,
          signal: result.signal,
          processError: unicodeProcessFailure(result.error),
        };
        writeFileSync(path.join(directory, `${stem}.process.json`), JSON.stringify(frame) + "\n", {
          flag: "wx",
        });
        frames.push(frame);
      }
    }
  assertSource(expected, consumer);
  assert.deepEqual(providers(consumer), before);
  const receipt: Receipt = {
    schema: "vize-public-unicode-v1",
    issue: 9493,
    version: publishedVersion,
    head: expected.head,
    source: expected,
    providers: before,
    frames,
    success: true,
  };
  // Validate the same complete receipt in the final seal, not native.success alone.
  assertPublishedUnicode(receipt, expected, publishedVersion, consumer, path.dirname(output));
  writeFileSync(
    path.join(path.dirname(output), "native-unicode.json"),
    JSON.stringify(receipt, null, 2) + "\n",
    { flag: "wx" },
  );
  return receipt;
}

export function assertPublishedUnicode(
  receipt: Receipt,
  expected: Source,
  publishedVersion: string,
  consumer: string,
  output: string,
) {
  assert.equal(receipt.schema, "vize-public-unicode-v1");
  assert.equal(receipt.issue, 9493);
  assert.equal(receipt.version, publishedVersion);
  assert.equal(receipt.head, expected.head);
  assert.deepEqual(receipt.source, expected);
  assert.equal(
    receipt.frames.length,
    20,
    "all five original cases, both modes, both providers required",
  );
  assertSource(expected, consumer);
  assert.deepEqual(receipt.providers, providers(consumer));
  assert.equal(receipt.success, true);
  const directory = path.join(output, "native-unicode");
  let index = 0;
  for (const production of [false, true])
    for (const entry of cases) {
      const compile = JSON.parse(
        readFileSync(path.join(directory, `${entry.name}-${production}.compile.json`), "utf8"),
      );
      assert.equal(compile.source, source(entry.template));
      assert.deepEqual(compile.options, options(production));
      assert.deepEqual(compile.result.errors, []);
      assert.deepEqual(compile.result.warnings, []);
      assert.equal(typeof compile.result.code, "string");
      assert.ok(compile.result.code.length > 0);
      const results = [];
      for (const official of [false, true]) {
        const frame = receipt.frames[index++],
          stem = `${entry.name}-${production}-${official}`;
        assert.deepEqual(
          JSON.parse(readFileSync(path.join(directory, `${stem}.process.json`), "utf8")),
          frame,
        );
        assert.deepEqual(
          {
            name: frame.name,
            production: frame.production,
            official: frame.official,
            stem: frame.stem,
          },
          { name: entry.name, production, official, stem },
        );
        assert.equal(frame.status, 0);
        assert.ok(Number.isSafeInteger(frame.pid) && frame.pid > 0);
        assert.equal(frame.signal, null);
        assert.equal(frame.processError, null);
        const input = readFileSync(path.join(directory, `${stem}.input.bin`)),
          stdout = readFileSync(path.join(directory, `${stem}.stdout.bin`)),
          stderr = readFileSync(path.join(directory, `${stem}.stderr.bin`));
        assert.equal(digest(input), frame.inputSha256);
        assert.equal(digest(stdout), frame.stdoutSha256);
        assert.equal(digest(stderr), frame.stderrSha256);
        assert.deepEqual(
          JSON.parse(input.toString()),
          frameInput(entry, production, official, compile.result.code),
        );
        const observations = stderr
          .toString("utf8")
          .split("\n")
          .filter((line) => line.startsWith('{"unicodeKeyScope":'))
          .map((line) => JSON.parse(line));
        assert.deepEqual(
          observations.map((value) => value.unicodeKeyScope),
          ["module", "key-values", "dom"],
        );
        const [module, keys, dom] = observations;
        assert.equal(module.authority, official ? "official-stock" : "fresh-native");
        assert.equal(module.source, compile.source);
        assert.equal(module.production, production);
        assert.equal(module.compilerVersion, version);
        assert.equal(module.vaporVersion, version);
        assert.equal(module.runtimeVersion, version);
        assert.equal(module.runtimePath, receipt.providers.files[production ? 4 : 3].file);
        if (!official) assert.equal(module.code, compile.result.code);
        assert.equal(module.keyFunctions.length, 1);
        assert.deepEqual(keys.values, [entry.value]);
        assert.deepEqual(dom.diagnostics, []);
        const trace = JSON.parse(stdout.toString());
        assert.equal(trace.length, 2);
        assert.deepEqual(
          trace[0].tree.map((node: { children: unknown[] }) => node.children[0]),
          entry.texts,
        );
        assert.deepEqual(trace[1].tree, []);
        results.push(trace);
      }
      assert.deepEqual(
        results[0],
        results[1],
        "whole native runtime trace must equal pinned stock",
      );
    }
}
