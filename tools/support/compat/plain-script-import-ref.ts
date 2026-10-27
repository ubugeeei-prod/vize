import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";

type Packet = Record<string, unknown> & {
  code: string;
  map?: string;
  errors: string[];
  warnings: string[];
};
type Native = {
  compileSfc(source: string, options: Record<string, unknown>): Packet;
  getNativeBuildToken?(): string;
};
type Manifest = {
  issue: string;
  bodySha256: string;
  files: { logical: string; stored: string; sha256: string }[];
};
const sha = (value: string | Uint8Array) => createHash("sha256").update(value).digest("hex");
const json = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n");
const require = createRequire(import.meta.url);
const root = resolve(process.env.GITHUB_WORKSPACE ?? process.cwd());
const fixtures = resolve(
  process.env.VIZE10903_FIXTURES ??
    join(root, "tests/_fixtures/differential/compiler/plain-script-import-10903"),
);
const ownFile = fileURLToPath(import.meta.url);
const fixtureManifest = JSON.parse(
  readFileSync(join(fixtures, "manifest.json"), "utf8"),
) as Manifest;

function identity(nativePath: string, proof: string) {
  const native = require(nativePath) as Native;
  const images = Object.keys(require.cache).filter(
    (path) => path.endsWith(".node") && path.includes("vize"),
  );
  assert.equal(images.length, 1, "the selected compiler must use one real native image");
  const image = realpathSync(images[0]!);
  const result: Record<string, unknown> = {
    nativePath: realpathSync(nativePath),
    image,
    imageSha256: sha(readFileSync(image)),
    token: native.getNativeBuildToken?.(),
    node: process.version,
    source: process.env.SOURCE_SHA ?? null,
  };
  if (process.env.VIZE10903_MODE === "source") {
    const receipt = JSON.parse(
      readFileSync(join(root, "npm/native/.artifacts/native/build-token-receipt.json"), "utf8"),
    );
    assert.equal(receipt.source, process.env.SOURCE_SHA);
    assert.equal(receipt.identity.token, native.getNativeBuildToken?.());
    assert.equal(receipt.binary.sha256, result.imageSha256);
    result.receipt = receipt;
  }
  json(join(proof, "native-identity.json"), result);
  return native;
}

function preload() {
  const proof = process.env.VIZE10903_CALL_PROOF!;
  mkdirSync(proof, { recursive: true });
  const native = identity(process.env.VIZE10903_NATIVE!, proof);
  const object = native as unknown as Record<string, (...args: unknown[]) => unknown>;
  let count = 0;
  for (const name of ["compileSfc", "compileSfcBatchWithResults"]) {
    if (typeof object[name] !== "function") continue;
    const original = object[name]!;
    object[name] = (...args: unknown[]) => {
      const start = performance.now();
      try {
        const result = original(...args);
        json(join(proof, "call-" + process.pid + "-" + count++ + ".json"), {
          name,
          args,
          result,
          durationMs: performance.now() - start,
        });
        return result;
      } catch (error) {
        json(join(proof, "call-" + process.pid + "-" + count++ + ".json"), {
          name,
          args,
          error: String(error),
          durationMs: performance.now() - start,
        });
        throw error;
      }
    };
  }
  process.on("exit", (code) => json(join(proof, "exit-" + process.pid + ".json"), { code, count }));
}

function command(
  executable: string,
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  proof: string,
) {
  const start = performance.now();
  const result = spawnSync(executable, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 240_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  json(proof, {
    executable,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
    durationMs: performance.now() - start,
  });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
  return result.stdout;
}

function validateMap(packet: Packet, source: string, filename: string, fromHost: NodeJS.Require) {
  assert.equal(typeof packet.map, "string", filename + " must retain a complete source map");
  const map = JSON.parse(packet.map!);
  assert.equal(map.version, 3);
  assert.deepEqual(map.sources, [filename]);
  assert.deepEqual(map.sourcesContent, [source]);
  assert.equal(typeof map.mappings, "string");
  assert(map.mappings.length > 0);
  const fromSfc = createRequire(fromHost.resolve("@vue/compiler-sfc/package.json"));
  const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
    "@jridgewell/sourcemap-codec",
  ) as { decode(value: string): number[][][]; encode(value: number[][][]): string };
  const decoded = codec.decode(map.mappings);
  assert.equal(codec.encode(decoded), map.mappings);
  const generatedLines = packet.code.split(/\r\n|[\r\n\u2028\u2029]/);
  const originalLines = source.split(/\r\n|[\r\n\u2028\u2029]/);
  let mapped = 0;
  decoded.forEach((line, index) => {
    let previous = -1;
    for (const segment of line) {
      const [column, sourceIndex, originalLine, originalColumn, nameIndex] = segment;
      assert(column! >= previous && column! <= generatedLines[index]!.length);
      previous = column!;
      if (segment.length === 1) continue;
      assert([4, 5].includes(segment.length));
      assert.equal(sourceIndex, 0);
      assert(originalLine! >= 0 && originalLine! < originalLines.length);
      assert(originalColumn! >= 0 && originalColumn! <= originalLines[originalLine!]!.length);
      if (segment.length === 5) assert(nameIndex! >= 0 && nameIndex! < map.names.length);
      mapped++;
    }
  });
  assert(mapped > 0);
}

async function matrix() {
  const mode = process.env.VIZE10903_MODE!;
  const expected = process.env.VIZE10903_EXPECT!;
  const publicVersion = process.env.VIZE10903_PUBLIC_VERSION ?? "0.589.0";
  assert(/^0\.\d+\.\d+$/.test(publicVersion));
  if (mode === "public" && expected === "before") assert.equal(publicVersion, "0.589.0");
  assert(["public", "source"].includes(mode));
  assert(["before", "after"].includes(expected));
  const proof = resolve(process.env.VIZE10903_PROOF!);
  mkdirSync(proof, { recursive: true });
  assert.equal(sha(readFileSync(join(fixtures, "issue-body.txt"))), fixtureManifest.bodySha256);
  const original = new Map<string, string>();
  for (const file of fixtureManifest.files) {
    const bytes = readFileSync(join(fixtures, file.stored), "utf8");
    assert.equal(sha(bytes), file.sha256, file.logical);
    original.set(file.logical, bytes);
  }
  assert.equal(original.size, 8);
  const host = join(proof, "host");
  mkdirSync(host, { recursive: true });
  json(join(host, "package.json"), {
    private: true,
    type: "module",
    dependencies: {
      vue: "3.5.41",
      "@vue/compiler-sfc": "3.5.41",
      "@vitejs/plugin-vue": "6.0.8",
      vite: "8.2.0",
      playwright: "1.62.1",
      ...(mode === "public"
        ? { "@vizejs/vite-plugin": publicVersion, "@vizejs/native": publicVersion }
        : {}),
    },
  });
  command(
    "npm",
    [
      "install",
      "--prefix",
      host,
      "--registry=https://registry.npmjs.org",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
    ],
    host,
    process.env,
    join(proof, "install.json"),
  );
  if (mode === "source") {
    mkdirSync(join(host, "node_modules/@vizejs"), { recursive: true });
    symlinkSync(
      join(root, "npm/builder/vite"),
      join(host, "node_modules/@vizejs/vite-plugin"),
      "dir",
    );
  }
  const fromHost = createRequire(join(host, "package.json"));
  for (const [name, version] of Object.entries({
    vue: "3.5.41",
    "@vue/compiler-sfc": "3.5.41",
    "@vitejs/plugin-vue": "6.0.8",
    vite: "8.2.0",
    playwright: "1.62.1",
  })) {
    assert.equal(fromHost(name + "/package.json").version, version);
  }
  if (mode === "public") {
    for (const name of ["@vizejs/native", "@vizejs/vite-plugin"]) {
      assert.equal(
        JSON.parse(readFileSync(join(host, "node_modules", name, "package.json"), "utf8")).version,
        publicVersion,
      );
    }
  }
  const nativePath =
    mode === "source" ? join(root, "npm/native/index.js") : fromHost.resolve("@vizejs/native");
  const native = identity(nativePath, proof);
  command(
    process.execPath,
    [
      join(
        dirname(fromHost.resolve("playwright/package.json")),
        fromHost("playwright/package.json").bin.playwright,
      ),
      "install",
      "--with-deps",
      "chromium",
    ],
    host,
    process.env,
    join(proof, "browser-install.json"),
  );
  const originalApp = original.get("src/App.vue")!;
  const cases: {
    name: string;
    app: string;
    files: Record<string, string>;
    kind: "unknown" | "constant";
  }[] = [
    { name: "original", app: originalApp, files: {} as Record<string, string>, kind: "unknown" },
    {
      name: "named-alias",
      app: originalApp
        .replace("{ label, onPing }", "{ label as alias, onPing as handler }")
        .replace(':text="label"', ':text="alias"')
        .replace('@ping="onPing"', '@ping="handler"'),
      files: {},
      kind: "unknown",
    },
    {
      name: "default-unknown",
      app: originalApp.replace(
        'import { label, onPing } from "./store";',
        'import label from "./label";\nimport onPing from "./listener";',
      ),
      files: {
        "src/label.ts": 'import { label } from "./store";\nexport default label;\n',
        "src/listener.ts": 'import { onPing } from "./store";\nexport default onPing;\n',
      },
      kind: "unknown",
    },
    {
      name: "constant-controls",
      app: originalApp
        .replace(
          'import { label, onPing } from "./store";',
          'import * as store from "./constants";\nimport { unref as identity } from "vue";\nimport Child from "./Child.vue";\nimport { default as DefaultAlias } from "./Child.vue";\nimport { "default" as StringDefaultAlias } from "./Child.vue";\nimport type { Hidden } from "./types";\nimport { type Marker, label, onPing } from "./store";',
        )
        .replace('import Child from "./Child.vue";\n</script>', "const ready = true;\n</script>")
        .replace('<Child :text="label"', '<DefaultAlias :text="store.label"')
        .replace('@ping="onPing"', '@ping="store.onPing"'),
      files: {
        "src/constants.ts":
          'export const label = "from ref";\nexport const onPing = () => { (window as unknown as { pings: number }).pings++; };\n',
        "src/types.ts": "export interface Hidden { value: string }\n",
      },
      kind: "constant",
    },
  ];
  const packets: unknown[] = [];
  const mapAbsences: unknown[] = [];
  const vue = fromHost("@vue/compiler-sfc");
  for (const entry of cases) {
    const files = new Map(original);
    files.set("src/App.vue", entry.app);
    if (entry.name === "constant-controls")
      files.set(
        "src/store.ts",
        files.get("src/store.ts")! + "\nexport interface Marker { value: string }\n",
      );
    for (const [name, content] of Object.entries(entry.files)) files.set(name, content);
    for (const [filename, source] of files) {
      if (!filename.endsWith(".vue")) continue;
      for (const inlineTemplate of [false, true]) {
        for (const sourceMap of [false, true]) {
          const options = { filename, inlineTemplate, sourceMap, isProd: inlineTemplate };
          const result = native.compileSfc(source, options);
          const parsed = vue.parse(source, { filename, sourceMap });
          assert.deepEqual(parsed.errors, []);
          const officialScript = vue.compileScript(parsed.descriptor, {
            id: "issue-10903",
            inlineTemplate,
            sourceMap,
            isProd: inlineTemplate,
          });
          const officialTemplate = inlineTemplate
            ? null
            : vue.compileTemplate({
                source: parsed.descriptor.template.content,
                filename,
                id: "issue-10903",
                sourceMap,
                compilerOptions: { bindingMetadata: officialScript.bindings, isTS: true },
              });
          packets.push({
            case: entry.name,
            filename,
            source,
            sourceSha256: sha(source),
            options,
            native: result,
            official: { script: officialScript, template: officialTemplate },
          });
          json(join(proof, "full-module-packets.json"), packets);
          assert.deepEqual(result.errors, []);
          if (sourceMap) {
            if (result.map === undefined) {
              const historicalPublic =
                mode === "public" && publicVersion === "0.589.0" && expected === "before";
              const originalMacroOnlyChild =
                filename === "src/Child.vue" && source === original.get("src/Child.vue");
              assert(
                historicalPublic || originalMacroOnlyChild,
                filename + " must retain its complete map",
              );
              mapAbsences.push({
                case: entry.name,
                filename,
                options,
                nativeVersion: mode === "public" ? publicVersion : null,
                source: process.env.SOURCE_SHA ?? null,
                sourceSha256: sha(source),
                observation: historicalPublic
                  ? "Historical public response omits map"
                  : "Exact original macro-only Child has no authored module provenance; complete DTO must match before/after",
              });
              json(join(proof, "map-absence-observations.json"), mapAbsences);
            } else validateMap(result, source, filename, fromHost);
          } else assert(!result.map, "sourceMap:false must stay absent");
          if (filename === "src/App.vue" && inlineTemplate && entry.kind === "unknown") {
            const names = entry.name === "named-alias" ? ["alias", "handler"] : ["label", "onPing"];
            for (const name of names) {
              assert.equal(officialScript.bindings[name], "setup-maybe-ref");
              assert(officialScript.content.includes("_unref(" + name + ")"));
              assert.equal(
                result.code.includes("_unref(" + name + ")"),
                expected === "after",
                entry.name + ":" + name,
              );
            }
          }
          if (filename === "src/App.vue" && entry.kind === "constant") {
            for (const name of ["store", "identity", "Child", "DefaultAlias", "StringDefaultAlias"])
              assert.equal(officialScript.bindings[name], "setup-const");
            for (const name of ["store", "identity", "Child", "DefaultAlias", "StringDefaultAlias"])
              assert(!result.code.includes("_unref(" + name + ")"), name + " must stay constant");
            assert(!result.code.includes("Hidden"));
            assert(!result.code.includes("Marker"));
          }
        }
      }
    }
    const project = join(proof, "projects", entry.name);
    mkdirSync(project, { recursive: true });
    for (const [filename, content] of files) {
      const path = join(project, filename);
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, content);
    }
    symlinkSync(join(host, "node_modules"), join(project, "node_modules"), "dir");
    // Preserve every authored source byte, including the original run.mjs.
    for (const file of fixtureManifest.files) {
      if (file.logical === "src/App.vue" && entry.name !== "original") continue;
      if (file.logical === "src/store.ts" && entry.name === "constant-controls") continue;
      assert.equal(sha(readFileSync(join(project, file.logical))), file.sha256);
    }
    for (const compiler of ["vize", "vue"]) {
      for (const modeName of ["build", "dev"]) {
        const callProof = join(
          proof,
          "actual-vite-native",
          entry.name + "-" + compiler + "-" + modeName,
        );
        mkdirSync(callProof, { recursive: true });
        const stdout = command(
          process.execPath,
          ["--import", pathToFileURL(ownFile).href + "?preload=1", "run.mjs"],
          project,
          {
            ...process.env,
            COMPILER: compiler,
            MODE: modeName,
            VIZE10903_NATIVE: nativePath,
            VIZE10903_CALL_PROOF: callProof,
          },
          join(proof, entry.name + "-" + compiler + "-" + modeName + ".json"),
        );
        const fails =
          compiler === "vize" &&
          modeName === "build" &&
          entry.kind === "unknown" &&
          expected === "before";
        const wanted =
          compiler +
          " (" +
          modeName +
          '): text "' +
          (fails ? "NOT A STRING: object" : "from ref") +
          '", pings ' +
          (fails ? 0 : 1) +
          ' (expected "from ref", 1)';
        assert(stdout.split("\n").includes(wanted), stdout);
        assert(!stdout.includes(" errors: "), stdout);
        if (compiler === "vize")
          assert(
            readdirSync(callProof).some((name) => name.startsWith("call-")),
            "the real Vite adapter must call the authenticated native object",
          );
      }
    }
  }
  const perfOptions = {
    filename: "src/App.vue",
    inlineTemplate: true,
    sourceMap: true,
    isProd: true,
  };
  for (let index = 0; index < 5; index++) native.compileSfc(originalApp, perfOptions);
  const durations = [];
  for (let index = 0; index < 100; index++) {
    const start = performance.now();
    native.compileSfc(originalApp, perfOptions);
    durations.push(performance.now() - start);
  }
  json(join(proof, "performance.json"), {
    source: process.env.SOURCE_SHA ?? null,
    inputSha256: sha(originalApp),
    options: perfOptions,
    warmup: 5,
    iterations: 100,
    durationsMs: durations,
    policy: "observation only; no speed threshold",
    identity: JSON.parse(readFileSync(join(proof, "native-identity.json"), "utf8")),
  });
  json(join(proof, "acceptance.json"), {
    mode,
    expected,
    issue: fixtureManifest.issue,
    bodySha256: fixtureManifest.bodySha256,
    completeNativeAndOfficialPackets: packets.length,
    browserRuns: cases.length * 4,
    source: process.env.SOURCE_SHA ?? null,
    pass: true,
  });
  console.log(
    JSON.stringify({
      mode,
      expected,
      packets: packets.length,
      browserRuns: cases.length * 4,
      pass: true,
    }),
  );
}

if (new URL(import.meta.url).searchParams.has("preload")) preload();
else await matrix();
