/** Exact-source comparison for #13558 and #13569 using the reporter's 1,000 SFCs. */
import assert from "node:assert/strict";
import { spawnSync, execFileSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const here = path.dirname(fileURLToPath(import.meta.url));
const worker = path.join(here, "imported-props-worker.mjs");
const generator = new URL(
  "../fixtures/client-vue-resolution-13795/generate.mjs.txt",
  import.meta.url,
);
const manifest = JSON.parse(fs.readFileSync(new URL("inputs.json", generator), "utf8"));
const generatorBytes = fs.readFileSync(generator);
assert.deepEqual(
  { bytes: generatorBytes.length, sha256: hash(generatorBytes) },
  manifest.files["generate.mjs.txt"],
);
const [baseInput, headInput, outInput] = process.argv.slice(2);
assert(baseInput && headInput && outInput);
const out = path.resolve(outInput);
assert(!fs.existsSync(out), "comparison output must be a fresh directory");
fs.mkdirSync(out, { recursive: true });
const report = {
  schema: "vize.importedPropsComparison.v1",
  status: "attempt",
  node: process.version,
  machine: {
    platform: process.platform,
    arch: process.arch,
    cpus: os.cpus(),
    availableParallelism: os.availableParallelism(),
  },
  fixture: { generatorSha256: hash(generatorBytes), components: 1000, environments: 3 },
  recipe: {
    profile: "ci-opt",
    features: "napi,legacy",
    threads: [1, 4, 16],
    chunks: [32, 128, 1000],
    rounds: 3,
  },
  sources: {},
  samples: [],
  vue: [],
};
const retain = () =>
  fs.writeFileSync(path.join(out, "report.json"), JSON.stringify(report, null, 2) + "\n");
retain();
function run(program, args, cwd, name, extra = {}) {
  const result = spawnSync(program, args, {
    cwd,
    env: { ...process.env, ...extra },
    encoding: "utf8",
    maxBuffer: 32 << 20,
  });
  fs.writeFileSync(path.join(out, name + ".log"), (result.stdout ?? "") + (result.stderr ?? ""));
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${name}: ${result.stderr?.slice(-3000)}`);
  return result.stdout;
}
function build(input, role, expected) {
  const root = fs.realpathSync(path.resolve(input));
  const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const sha = git("rev-parse", "HEAD");
  assert.match(sha, /^[0-9a-f]{40}$/u);
  if (expected) assert.equal(sha, expected);
  assert.equal(git("diff", "--name-only", "HEAD"), "", "source must match the committed tree");
  const token = randomBytes(16).toString("hex");
  const target = path.join(root, "target/imported-props");
  const args = [
    "build",
    "--locked",
    "--profile",
    "ci-opt",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
  ];
  const source = {
    root,
    sha,
    tree: git("rev-parse", "HEAD^{tree}"),
    token,
    target,
    cargoArgs: args,
    rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    lockSha256: hash(fs.readFileSync(path.join(root, "Cargo.lock"))),
  };
  report.sources[role] = source;
  retain();
  run("cargo", args, root, `build-${role}`, {
    CARGO_TARGET_DIR: target,
    VIZE_NATIVE_BUILD_TOKEN: token,
  });
  const name =
    process.platform === "win32"
      ? "vize_vitrine.dll"
      : process.platform === "darwin"
        ? "libvize_vitrine.dylib"
        : "libvize_vitrine.so";
  const image = path.join(out, `${role}.node`);
  fs.copyFileSync(path.join(target, "ci-opt", name), image);
  source.image = image;
  source.imageSha256 = hash(fs.readFileSync(image));
  retain();
  return image;
}
const images = {
  base: build(baseInput, "base", process.env.IMPORTED_PROPS_BASE_SHA),
  head: build(headInput, "head", process.env.IMPORTED_PROPS_HEAD_SHA),
};
assert.equal(report.sources.base.lockSha256, report.sources.head.lockSha256);
const provider = path.join(out, "provider");
run(
  process.platform === "win32" ? "npm.cmd" : "npm",
  [
    "install",
    "--prefix",
    provider,
    "--save-exact",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "@vue/compiler-sfc@3.5.46",
    "vue@3.5.46",
    "typescript@6.0.3",
  ],
  out,
  "install-oracle",
);
report.oracleLockSha256 = hash(fs.readFileSync(path.join(provider, "package-lock.json")));
const fixture = path.join(out, "fixture");
fs.mkdirSync(fixture);
fs.writeFileSync(path.join(out, "generate.mjs"), generatorBytes);
const outputs = new Map();
for (const [mode, modules] of [
  ["inline", 1],
  ["local", 1],
  ["barrel", 200],
  ["barrel", 800],
]) {
  const shape = `${mode}-${modules}`;
  run(
    process.execPath,
    [path.join(out, "generate.mjs"), "1000", "0", mode, String(modules)],
    fixture,
    `generate-${shape}`,
  );
  fs.writeFileSync(
    path.join(fixture, "tsconfig.json"),
    '{"compilerOptions":{"moduleResolution":"bundler","strict":true},"include":["src"]}\n',
  );
  const dependencies =
    mode === "inline"
      ? []
      : [
          "tsconfig.json",
          "src/tsconfig.json",
          "src/components/tsconfig.json",
          "src/types.ts",
          ...(mode === "barrel"
            ? [
                "src/models/index.ts",
                ...Array.from({ length: modules }, (_, index) => `src/models/Model${index}.ts`),
              ]
            : []),
        ]
          .map((name) => path.join(fixture, name))
          .sort();
  fs.writeFileSync(path.join(fixture, "dependencies.json"), JSON.stringify(dependencies));
  const inventory = fs
    .readdirSync(path.join(fixture, "src"), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => {
      const file = path.join(entry.parentPath, entry.name);
      const bytes = fs.readFileSync(file);
      return { path: path.relative(fixture, file), bytes: bytes.length, sha256: hash(bytes) };
    })
    .sort((a, b) => a.path.localeCompare(b.path));
  fs.writeFileSync(path.join(out, `inputs-${shape}.json`), JSON.stringify(inventory, null, 2));
  for (let round = 0; round < report.recipe.rounds; round++) {
    const oracleReceipt = path.join(out, `vue-${shape}-${round}.json`);
    run(
      process.execPath,
      [worker, "vue", provider, fixture, "1000", oracleReceipt],
      out,
      `vue-${shape}-${round}`,
    );
    report.vue.push({
      mode,
      modules,
      round,
      ...JSON.parse(fs.readFileSync(oracleReceipt, "utf8")),
    });
    for (const threads of round % 2 ? [16, 4, 1] : [1, 4, 16]) {
      for (const chunk of round % 2 ? [1000, 128, 32] : [32, 128, 1000]) {
        for (const side of (round + threads + chunk) % 2 ? ["base", "head"] : ["head", "base"]) {
          const name = `${side}-${shape}-t${threads}-c${chunk}-r${round}`;
          const receipt = path.join(out, name + ".json");
          run(
            process.execPath,
            [worker, "native", images[side], fixture, String(chunk), receipt],
            out,
            name,
            { RAYON_NUM_THREADS: String(threads) },
          );
          const sample = JSON.parse(fs.readFileSync(receipt, "utf8"));
          assert.equal(sample.bindingSha256, report.sources[side].imageSha256);
          assert.equal(sample.buildToken, report.sources[side].token);
          const expected = outputs.get(shape) ?? sample.cold.output.sha256;
          assert.equal(
            sample.cold.output.sha256,
            expected,
            "all threads, chunks and source images must return identical complete results",
          );
          outputs.set(shape, expected);
          report.samples.push({
            mode,
            modules,
            round,
            threads,
            chunk,
            side,
            receipt: path.basename(receipt),
            cold: {
              wallMs: sample.cold.wallMs,
              cpuMs: sample.cold.cpuMs,
              calls: sample.cold.calls,
            },
            warm: {
              wallMs: sample.warm.wallMs,
              cpuMs: sample.warm.cpuMs,
              calls: sample.warm.calls,
            },
          });
          retain();
          process.stdout.write(
            `${name}: ${sample.cold.wallMs.toFixed(1)}ms wall, ${sample.cold.cpuMs.toFixed(1)}ms CPU\n`,
          );
        }
      }
    }
  }
}
report.status = "measured";
retain();
const lines = [
  "| props | threads | chunk | source | cold wall ms | cold CPU ms | warm wall ms | warm CPU ms |",
  "| :-- | --: | --: | :-- | --: | --: | --: | --: |",
];
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
for (const mode of ["inline", "local", "barrel"]) {
  for (const modules of mode === "barrel" ? [200, 800] : [1]) {
    for (const threads of report.recipe.threads) {
      for (const chunk of report.recipe.chunks) {
        for (const side of ["base", "head"]) {
          const rows = report.samples.filter(
            (row) =>
              row.mode === mode &&
              row.modules === modules &&
              row.threads === threads &&
              row.chunk === chunk &&
              row.side === side,
          );
          lines.push(
            `| ${mode}-${modules} | ${threads} | ${chunk} | ${side} | ${["cold", "warm"].flatMap((phase) => ["wallMs", "cpuMs"].map((metric) => median(rows.map((row) => row[phase][metric])).toFixed(1))).join(" | ")} |`,
          );
        }
      }
    }
  }
}
fs.writeFileSync(path.join(out, "summary.md"), lines.join("\n") + "\n");
if (process.env.GITHUB_STEP_SUMMARY)
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, lines.join("\n") + "\n");
