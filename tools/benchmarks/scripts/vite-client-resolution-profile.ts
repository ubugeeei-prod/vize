/** Diagnostic experiment for #13795 (Q16solver); no product acceptance credit. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

const edits = [
  [
    "return entry ? { id: entry } : null;",
    "return typeof entry === 'string' ? { id: entry } : entry;",
  ],
  [
    'return (await oncePerBuild(ctx, state, [\n\t\t"project-vue-runtime",',
    'const resolved = await oncePerBuild(ctx, state, [\n\t\t"project-vue-runtime",',
  ],
  [
    "}))?.id ?? null;",
    "});\n\treturn resolved && Object.keys(resolved).length === 1 ? resolved.id : resolved;",
  ],
  [
    "state.logger.log(`resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`);\n\t\t\t\treturn projectLocalEntry;",
    "state.logger.log(`resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`);\n\t\t\t\treturn resolved.id === projectLocalEntry ? resolved : path.normalize(resolved.id) === path.normalize(projectLocalEntry) ? { ...resolved, id: projectLocalEntry } : projectLocalEntry;",
  ],
] as const;

/** Fail closed if the published experiment no longer has these exact source spans. */
export function metadataVariant(source: string): string {
  for (const [before, after] of edits) {
    assert.equal(source.split(before).length, 2, `unique published experiment span: ${before}`);
    source = source.replace(before, () => after);
  }
  return source;
}

const hash = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
export function reportInput(
  name:
    | "vite.config.ts.txt"
    | "generate.mjs.txt"
    | "count-native.mjs.txt"
    | "tsconfig.json.txt"
    | "index.html.txt",
): string {
  const root = new URL("../fixtures/client-vue-resolution-13795/", import.meta.url);
  const manifest = JSON.parse(readFileSync(new URL("inputs.json", root), "utf8")) as {
    schema: string;
    credit: { compiled: boolean; native: boolean; performance: boolean };
    files: Record<string, { bytes: number; sha256: string }>;
  };
  assert.equal(manifest.schema, "vize.vueClientResolutionReportInputs.v1");
  assert.deepEqual(manifest.credit, { compiled: false, native: false, performance: false });
  assert.ok(Object.hasOwn(manifest.files, name));
  const text = readFileSync(new URL(name, root), "utf8");
  assert.deepEqual({ bytes: Buffer.byteLength(text), sha256: hash(text) }, manifest.files[name]);
  return text;
}
type Sample = {
  round: number;
  mode: string;
  count: number;
  ms: number;
  outputs: Record<string, string>;
  ssr: string;
  native: { calls: number; files: number };
  resolution: { calls: number; claimed: number; object: number; first: unknown };
};

function main() {
  const out = path.resolve(
    process.env.PROFILE_OUTPUT ?? mkdtempSync(path.join(tmpdir(), "vize-vue-profile-")),
  );
  mkdirSync(out, { recursive: true });
  const root = path.join(out, "consumer");
  assert.ok(!existsSync(root), "each experiment needs a fresh consumer");
  mkdirSync(root);
  writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ private: true, type: "module", packageManager: "pnpm@12.1.0" }),
  );
  const env = {
    ...process.env,
    RAYON_NUM_THREADS: "8",
    PROFILE_METRICS: path.join(out, "metrics.json"),
    PROFILE_NATIVE: path.join(out, "native.json"),
    TIMINGS_FILE: path.join(out, "timings.jsonl"),
  };
  function run(
    args: string[],
    log: string,
    extra: Record<string, string> = {},
    program = process.execPath,
  ) {
    const result = spawnSync(program, args, {
      cwd: root,
      env: { ...env, ...extra },
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    });
    writeFileSync(path.join(out, log), result.stdout + result.stderr);
    assert.ifError(result.error);
    assert.equal(result.status, 0, `${log}: ${result.stderr.slice(-2000)}`);
    return result.stdout;
  }
  const npm =
    process.env.npm_execpath ??
    path.join(
      path.dirname(process.execPath),
      process.platform === "win32"
        ? "node_modules/npm/bin/npm-cli.js"
        : "../lib/node_modules/npm/bin/npm-cli.js",
    );
  assert.ok(existsSync(npm), `npm CLI beside the selected Node: ${npm}`);
  run(
    [
      npm,
      "install",
      "--prefix",
      path.join(out, "tooling"),
      "--save-exact",
      "--no-fund",
      "--no-audit",
      "pnpm@12.1.0",
    ],
    "install-pnpm.log",
  );
  const pnpmRoot = path.join(out, "tooling/node_modules/pnpm");
  const pnpmManifest = JSON.parse(readFileSync(path.join(pnpmRoot, "package.json"), "utf8")) as {
    bin: { pnpm: string };
  };
  const pnpm = path.join(pnpmRoot, pnpmManifest.bin.pnpm);
  assert.ok(pnpm.startsWith(pnpmRoot + path.sep) && existsSync(pnpm));
  const installArgs = [
    "add",
    "--save-exact",
    "--ignore-scripts",
    "@vizejs/vite-plugin@0.809.0",
    "@vizejs/native@0.809.0",
    "vue@3.5.46",
    "vite@8.4.0",
    "@vitejs/plugin-vue@6.0.11",
    "typescript@6.0.3",
  ];
  if (/\.[cm]?js$/.test(pnpm)) run([pnpm, ...installArgs], "install.log");
  else run(installArgs, "install.log", {}, pnpm);
  const dist = path.join(root, "node_modules/@vizejs/vite-plugin/dist");
  const candidates = readdirSync(dist).filter(
    (file) =>
      file.endsWith(".mjs") &&
      readFileSync(path.join(dist, file), "utf8").includes(
        "async function resolveProjectVueRuntimeFrom(",
      ),
  );
  assert.equal(candidates.length, 1);
  const chunk = path.join(dist, candidates[0]!);
  assert.ok(readFileSync(path.join(dist, "index.mjs"), "utf8").includes(candidates[0]!));
  const original = readFileSync(chunk, "utf8");
  assert.equal(
    hash(original),
    "0f090dd79c0db778952925ccdda57473387b2c01113a8d5c1e18b207db59a37c",
    "exact published 0.809.0 producer",
  );
  const candidate = metadataVariant(original);
  const identity = {
    platform: process.platform,
    node: process.version,
    rayonThreads: 8,
    reportInputs: JSON.parse(
      readFileSync(
        new URL("../fixtures/client-vue-resolution-13795/inputs.json", import.meta.url),
        "utf8",
      ),
    ),
    chunk: candidates[0],
    baseSha256: hash(original),
    candidateSha256: hash(candidate),
    interpretation:
      "Prototype only: metadata is reused per directory. Custom importer-specific resolver metadata is not qualified. No production/native/performance acceptance credit.",
  };
  writeFileSync(path.join(out, "identity.json"), JSON.stringify(identity, null, 2));
  writeFileSync(path.join(out, "prototype-before.mjs"), original);
  writeFileSync(path.join(out, "prototype-after.mjs"), candidate);
  const observer = `import vize from '@vizejs/vite-plugin';
import {writeFileSync} from 'node:fs';
const mode = process.env.PROFILE_MODE;
export default function(...args) {
const plugins = [vize(...args)].flat(Infinity).filter(Boolean);
const counts = { calls:0, claimed:0, object:0, first:null };
for (const plugin of plugins) {
  const hook = plugin.resolveId;
  const handler = typeof hook === 'function' ? hook : hook?.handler;
  if (!handler || !plugin.name.startsWith('vite-plugin-vize')) continue;
  const wrapped = async function(id, importer, ...rest) {
    const observed = this.environment?.name === 'client' && id === 'vue' && importer?.includes('.vue');
    if (observed) counts.calls++;
    const result = observed && mode === 'defer' ? null : await handler.call(this, id, importer, ...rest);
    if (observed && result != null) {
      counts.claimed++;
      if (typeof result === 'object') counts.object++;
      counts.first ??= result;
    }
    return result;
  };
  plugin.resolveId = typeof hook === 'function' ? wrapped : {...hook, handler: wrapped};
}
plugins.push({name:'resolution-profile',closeBundle(){if(this.environment?.name === 'client') writeFileSync(process.env.PROFILE_METRICS, JSON.stringify(counts));}});
return plugins;
}`;
  writeFileSync(path.join(root, "profile-vize.mjs"), observer);
  const originalConfig = reportInput("vite.config.ts.txt");
  const importLine = 'import vize from "@vizejs/vite-plugin";';
  assert.equal(originalConfig.split(importLine).length, 2);
  writeFileSync(
    path.join(root, "vite.config.ts"),
    originalConfig.replace(importLine, 'import vize from "./profile-vize.mjs";'),
  );
  writeFileSync(path.join(root, "generate.mjs"), reportInput("generate.mjs.txt"));
  writeFileSync(path.join(root, "count-native.mjs"), reportInput("count-native.mjs.txt"));
  writeFileSync(
    path.join(root, "native-observer.mjs"),
    `import {writeFileSync} from 'node:fs';
if (process.env.COMPILER !== 'vue') await import('./count-native.mjs');
process.on('exit',()=>writeFileSync(process.env.PROFILE_NATIVE,JSON.stringify(globalThis.__nativeCompiles ?? {calls:0,files:0})));`,
  );
  writeFileSync(path.join(root, "index.html"), reportInput("index.html.txt"));
  writeFileSync(path.join(root, "tsconfig.json"), reportInput("tsconfig.json.txt"));
  const rows: Sample[] = [];
  const outputIdentity = new Map<string, Record<string, string>>();
  const ssrIdentity = new Map<number, string>();
  const nativeIdentity = new Map<number, Sample["native"]>();
  const save = () =>
    writeFileSync(
      path.join(out, "samples.json"),
      JSON.stringify({ identity, excludedWarmupRound: 0, rows }, null, 2),
    );
  function outputs(directory: string, prefix = ""): Record<string, string> {
    const result: Record<string, string> = {};
    for (const file of readdirSync(directory, { withFileTypes: true })) {
      const relative = `${prefix}${file.name}`;
      if (file.isDirectory())
        Object.assign(result, outputs(path.join(directory, file.name), `${relative}/`));
      else result[relative] = hash(readFileSync(path.join(directory, file.name)));
    }
    return result;
  }
  try {
    for (const count of [1, 1000]) {
      run(["generate.mjs", String(count), "0", "inline", "1"], `generate-${count}.log`);
      for (let round = 0; round < 6; round++) {
        const modes = [
          "base",
          "base-twin",
          "metadata",
          "metadata-twin",
          "defer",
          "defer-twin",
          "vue",
          "vue-twin",
        ];
        if (round % 2) modes.reverse();
        for (const mode of modes) {
          const variant = mode.replace("-twin", "");
          writeFileSync(chunk, variant === "metadata" ? candidate : original);
          for (const relative of ["dist", "node_modules/.vize", "node_modules/.vite"])
            rmSync(path.join(root, relative), { force: true, recursive: true });
          for (const file of ["metrics.json", "native.json", "timings.jsonl"])
            rmSync(path.join(out, file), { force: true });
          const began = performance.now();
          run(
            [
              "--import",
              "./native-observer.mjs",
              "node_modules/vite/bin/vite.js",
              "build",
              "--app",
            ],
            `build-${count}-${round}-${mode}.log`,
            { PROFILE_MODE: variant, COMPILER: variant === "vue" ? "vue" : "vize" },
          );
          const ms = performance.now() - began;
          const resolution =
            variant === "vue"
              ? { calls: 0, claimed: 0, object: 0, first: null }
              : (JSON.parse(
                  readFileSync(path.join(out, "metrics.json"), "utf8"),
                ) as Sample["resolution"]);
          const native = JSON.parse(
            readFileSync(path.join(out, "native.json"), "utf8"),
          ) as Sample["native"];
          if (variant !== "vue") {
            assert.deepEqual(
              native,
              count === 1 ? { calls: 2, files: 2 } : { calls: 8, files: 1000 },
              "pinned build: dual-target batches share the 1000 inputs; standalone target calls handle one SFC",
            );
            if (nativeIdentity.has(count)) assert.deepEqual(native, nativeIdentity.get(count));
            else nativeIdentity.set(count, native);
          }
          if (existsSync(path.join(out, "timings.jsonl")))
            writeFileSync(
              path.join(out, `timings-${count}-${round}-${mode}.jsonl`),
              readFileSync(path.join(out, "timings.jsonl")),
            );
          if (variant === "base" || variant === "metadata")
            assert.equal(
              resolution.claimed,
              count,
              "the report's pnpm redirect must actually run for every SFC",
            );
          if (variant === "metadata")
            assert.equal(
              resolution.object,
              count,
              "the metadata prototype must actually return native metadata",
            );
          if (variant === "defer") assert.equal(resolution.claimed, 0);

          const files = outputs(path.join(root, "dist"));
          const family = `${count}-${variant === "vue" ? "vue" : "vize"}`;
          if (outputIdentity.has(family))
            assert.deepEqual(
              files,
              outputIdentity.get(family),
              "complete build output must stay byte-identical",
            );
          else outputIdentity.set(family, files);
          const html = run(
            [
              "--input-type=module",
              "-e",
              "import {render} from './dist/server/entry-server.js';process.stdout.write(await render());",
            ],
            `ssr-${count}-${round}-${mode}.log`,
          );
          assert.equal((html.match(/<div>/g) ?? []).length, count, "every SFC must render");
          if (ssrIdentity.has(count))
            assert.equal(
              hash(html),
              ssrIdentity.get(count),
              "reference and variants must render identical HTML",
            );
          else ssrIdentity.set(count, hash(html));
          rows.push({
            round,
            mode,
            count,
            ms,
            outputs: files,
            ssr: hash(html),
            native,
            resolution,
          });
          save();
          console.log(JSON.stringify({ round, mode, count, ms }));
        }
      }
    }
  } finally {
    writeFileSync(chunk, original);
    assert.equal(hash(readFileSync(chunk)), identity.baseSha256);
    save();
  }
  // Keep the lock and original reports, not a node_modules artifact.
  rmSync(path.join(root, "node_modules"), { recursive: true, force: true });
  rmSync(path.join(out, "tooling"), { recursive: true, force: true });
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href)
  main();
