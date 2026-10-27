// Before-only observer. No product verdict or source/native build.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
type Obj = Record<string, unknown>;
const object = (value: unknown): Obj => {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Obj;
};
const text = (value: unknown): string => {
  assert.equal(typeof value, "string");
  return value as string;
};
const array = (value: unknown): unknown[] => {
  assert.ok(Array.isArray(value));
  return value;
};
const sha = (value: string | Buffer): string => createHash("sha256").update(value).digest("hex");
const save = (directory: string, name: string, value: unknown): void =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n");
const [workspaceArg, outputArg, corpusArg] = process.argv.slice(2);
assert.ok(workspaceArg && outputArg && corpusArg && process.argv.length === 5);
assert.ok([workspaceArg, outputArg, corpusArg].every(path.isAbsolute));
assert.equal(process.env.NODE_OPTIONS ?? "", "");
assert.equal(process.env.VIZE_NUXT_NATIVE_CUSTODY, undefined);
const workspace = fs.realpathSync(workspaceArg),
  out = path.resolve(outputArg);
fs.mkdirSync(out, { recursive: true });
const require = createRequire(path.join(workspace, "package.json"));
const owned = (file: string): string => {
  const result = fs.realpathSync(file);
  assert.ok(result.startsWith(workspace + path.sep));
  return result;
};
const version = "0.602.0";
const packages = ["vize", "@vizejs/native", "vue", "typescript"].map((name) => {
  const file = owned(require.resolve(name + "/package.json")),
    bytes = fs.readFileSync(file);
  const manifest = object(JSON.parse(bytes.toString()));
  assert.equal(manifest.name, name);
  assert.equal(
    manifest.version,
    name === "vue" ? "3.5.43" : name === "typescript" ? "6.0.3" : version,
  );
  return { name, file, manifest, sha256: sha(bytes) };
});
const cli = owned(path.join(workspace, "node_modules/.bin/vize"));
assert.equal(
  owned(createRequire(cli).resolve("@vizejs/native")),
  owned(require.resolve("@vizejs/native")),
);
const native: unknown = require("@vizejs/native");
assert.equal(typeof object(native).parseSfc, "function");
assert.equal(typeof object(native).analyzeSfc, "function");
const binaries = Object.keys(require.cache)
  .filter((file) => file.endsWith(".node"))
  .map((file) => {
    const physical = owned(file),
      bytes = fs.readFileSync(physical);
    return { file: physical, bytes: bytes.length, sha256: sha(bytes) };
  });
assert.equal(binaries.length, 1);
const tag = spawnSync(
  "git",
  ["ls-remote", "https://github.com/ubugeeei-prod/vize.git", "refs/tags/v" + version],
  { encoding: "utf8", timeout: 120000 },
);
save(out, "tag-identity.json", {
  status: tag.status,
  stdout: tag.stdout,
  stderr: tag.stderr,
  error: tag.error?.message ?? null,
});
assert.equal(tag.error, undefined);
assert.equal(tag.status, 0);
const [candidate, tagRef] = tag.stdout.trim().split(/\s+/);
assert.ok(candidate);
assert.match(candidate, /^[a-f0-9]{40}$/);
assert.equal(tagRef, "refs/tags/v" + version);
const receiptUrl =
  "https://github.com/ubugeeei-prod/vize/releases/download/v" +
  version +
  "/vize-" +
  process.platform +
  "-" +
  process.arch +
  ".vsix.receipt.json";
const response = await fetch(receiptUrl);
assert.equal(response.status, 200);
const publicReceipt = object(await response.json());
assert.equal(publicReceipt.source, candidate);
assert.equal(publicReceipt.version, version);
const images = array(object(publicReceipt.provider).files)
  .map(object)
  .filter((item) => text(item.path).endsWith(".node"));
assert.equal(images.length, 1);
assert.equal(images[0]?.sha256, binaries[0]?.sha256);
assert.equal(images[0]?.bytes, binaries[0]?.bytes);
const lock = fs.readFileSync(path.join(workspace, "package-lock.json"));
fs.writeFileSync(path.join(out, "package-lock.json"), lock);
save(out, "provider-identity.json", {
  stage: "before-only",
  version,
  candidate,
  packages,
  binaries,
  receiptUrl,
  publicReceipt,
  lockSha256: sha(lock),
  observerHead: process.env.GITHUB_SHA,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
});
const capture = (args: string[], cwd: string, artifacts: string, name: string) => {
  const started = performance.now();
  const result = spawnSync(cli, args, {
    cwd,
    env: { ...process.env, NO_COLOR: "1", LANG: "C", LC_ALL: "C" },
    timeout: 120000,
    maxBuffer: 16 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(artifacts, name + ".stdout.bin"), result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(path.join(artifacts, name + ".stderr.bin"), result.stderr ?? Buffer.alloc(0));
  const processPacket = {
    cli,
    cliSha256: sha(fs.readFileSync(cli)),
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    wallMs: performance.now() - started,
  };
  save(artifacts, name + "-process.json", processPacket);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  return { ...processPacket, stdout: result.stdout.toString(), stderr: result.stderr.toString() };
};
const cliVersion = capture(["--version"], workspace, out, "cli-version");
assert.equal(cliVersion.status, 0);
assert.equal(cliVersion.stdout.trim(), "vize " + version);
interface StockTs {
  version: string;
  ModuleKind: { CommonJS: number };
  ScriptTarget: { ES2022: number };
  transpileModule(
    source: string,
    options: unknown,
  ): { outputText: string; diagnostics?: unknown[] };
}
interface StockVue {
  isRef(value: unknown): boolean;
  isReactive(value: unknown): boolean;
  effect(callback: () => void): unknown;
  stop(runner: unknown): void;
}
const ts = require("typescript") as StockTs,
  vue = require("vue") as StockVue;
assert.equal(ts.version, "6.0.3");
for (const name of ["isRef", "isReactive", "effect", "stop"])
  assert.equal(typeof object(vue)[name], "function");
const raw = fs.readFileSync(corpusArg),
  corpus = object(JSON.parse(raw.toString()));
assert.equal(corpus.schema, "vize.public-composable-provide-before");
assert.equal(corpus.version, 1);
fs.writeFileSync(path.join(out, "authored-corpus.json"), raw);
const newCases = array(corpus.cases).map(object),
  oldCases = array(object(corpus.existingAssertionControls).cases).map(object);
assert.equal(newCases.length, 14);
assert.equal(oldCases.length, 27);
const rows: unknown[] = [],
  failures: { id: string; error: string }[] = [];
let cliPackets = 0,
  inspectorPackets = 0,
  publicNativePackets = 0,
  runtimeProbes = 0;
const probeRuntime = (row: Obj, inputs: Obj, artifacts: string): unknown => {
  const spec = object(row.runtime),
    directory = path.join(workspace, "runtime", text(row.id));
  fs.mkdirSync(directory, { recursive: true });
  const modules = new Map<string, Obj>();
  for (const field of ["selectionModule", "toastModule"]) {
    const sourceName = text(spec[field]);
    if (modules.has(sourceName)) continue;
    const source = text(inputs[sourceName]);
    const compilerOptions = {
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2022,
      esModuleInterop: true,
    };
    const compiled = ts.transpileModule(source, {
      compilerOptions,
      reportDiagnostics: true,
      fileName: sourceName,
    });
    assert.deepEqual(compiled.diagnostics ?? [], []);
    const file = path.join(directory, path.basename(sourceName, ".ts") + ".cjs");
    fs.writeFileSync(file, compiled.outputText);
    save(artifacts, path.basename(sourceName) + "-stock-transpilation.json", {
      sourceName,
      source,
      sourceSha256: sha(source),
      compilerOptions,
      output: compiled.outputText,
      outputSha256: sha(compiled.outputText),
      diagnostics: compiled.diagnostics ?? [],
    });
    modules.set(sourceName, object(createRequire(file)(file)));
  }
  const selectionModule = modules.get(text(spec.selectionModule)),
    toastModule = modules.get(text(spec.toastModule));
  assert.ok(selectionModule && toastModule);
  const selectionFactory = selectionModule[text(spec.selectionExport)],
    toastFactory = toastModule[text(spec.toastExport)];
  assert.equal(typeof selectionFactory, "function");
  assert.equal(typeof toastFactory, "function");
  const selectionResult: unknown = (selectionFactory as () => unknown)();
  const selected: unknown =
    spec.selectionMode === "direct" ? selectionResult : object(selectionResult).selected;
  let selectedEffectRuns = 0,
    selectedObserved: unknown = null;
  const readSelected = () => {
    const value = vue.isRef(selected) ? object(selected).value : selected;
    return Array.isArray(value) ? value.length : null;
  };
  const selectedRunner = vue.effect(() => {
    selectedEffectRuns++;
    selectedObserved = readSelected();
  });
  const selectedBefore = selectedObserved;
  if (vue.isRef(selected)) object(selected).value = ["probe"];
  else if (Array.isArray(selected)) selected.push("probe");
  const selectedAfterSet = {
    observed: selectedObserved,
    actual: readSelected(),
    effectRuns: selectedEffectRuns,
  };
  const clear = spec.selectionMode === "direct" ? undefined : object(selectionResult).clear;
  if (typeof clear === "function") clear();
  const selectedAfterClear = {
    observed: selectedObserved,
    actual: readSelected(),
    effectRuns: selectedEffectRuns,
  };
  vue.stop(selectedRunner);
  const toastProbe = (args: unknown[]) => {
    const toast = object((toastFactory as (...args: unknown[]) => unknown)(...args));
    let effectRuns = 0,
      observed: unknown = null;
    const read = () => {
      const member = toast.message;
      return member !== null && typeof member === "object" ? (object(member).value ?? null) : null;
    };
    const runner = vue.effect(() => {
      effectRuns++;
      observed = read();
    });
    const before = observed,
      show = toast.show;
    assert.equal(typeof show, "function");
    (show as (message: string) => void)("probe");
    const after = { observed, actual: read(), effectRuns };
    vue.stop(runner);
    return {
      args,
      objectIsReactive: vue.isReactive(toast),
      messageIsRef: vue.isRef(toast.message),
      before,
      after,
    };
  };
  const result = {
    authority:
      "actual pinned Vue runtime executing the exact source after independent stock TypeScript transpilation",
    selected: {
      isRef: vue.isRef(selected),
      isReactive: vue.isReactive(selected),
      before: selectedBefore,
      afterSet: selectedAfterSet,
      afterClear: selectedAfterClear,
    },
    toast: toastProbe([]),
    alternateToast:
      spec.toastAlternateArgs === undefined ? null : toastProbe(array(spec.toastAlternateArgs)),
  };
  const expected = object(row.runtimeExpected);
  assert.equal(result.selected.isRef, expected.selectedIsRef);
  assert.equal(result.toast.messageIsRef, expected.toastMessageIsRef);
  assert.equal(result.alternateToast?.messageIsRef ?? null, expected.alternateToastMessageIsRef);
  if (result.toast.messageIsRef) {
    assert.equal(result.toast.after.observed, "probe");
    assert.equal(result.toast.after.effectRuns, 2);
  } else assert.equal(result.toast.after.effectRuns, 1);
  save(artifacts, "stock-vue-runtime.json", result);
  return result;
};
for (const [kind, cases] of [
  ["composable", newCases],
  ["assertion", oldCases],
] as const) {
  for (const row of cases) {
    const id = kind + "-" + text(row.id),
      artifacts = path.join(out, id),
      project = path.join(workspace, "cases", id);
    fs.mkdirSync(artifacts);
    fs.mkdirSync(project, { recursive: true });
    const inputs = object(row.files),
      inputPins: Obj = {};
    for (const [name, value] of Object.entries(inputs)) {
      assert.ok(name.startsWith("src/") && !name.split("/").includes(".."));
      const source = text(value);
      if (kind === "assertion") assert.equal(sha(source), object(row.fileSha256)[name]);
      fs.mkdirSync(path.dirname(path.join(project, name)), { recursive: true });
      fs.writeFileSync(path.join(project, name), source);
      inputPins[name] = { bytes: Buffer.byteLength(source), sha256: sha(source) };
    }
    save(artifacts, "source-pins.json", inputPins);
    fs.cpSync(path.join(project, "src"), path.join(artifacts, "fixture/src"), { recursive: true });
    try {
      const doctor = capture(["doctor", "--format", "json", "."], project, artifacts, "doctor");
      const lint = capture(
        ["lint", "-f", "json", "--no-config", "--cross-file", "src/**/*.vue"],
        project,
        artifacts,
        "lint",
      );
      const doctorPacket: unknown = JSON.parse(doctor.stdout),
        lintPacket: unknown = JSON.parse(lint.stdout);
      save(artifacts, "doctor.json", doctorPacket);
      save(artifacts, "lint.json", lintPacket);
      cliPackets += 2;
      const inspector = capture(
        ["inspector", "--format", "json", "src/**/*.vue"],
        project,
        artifacts,
        "inspector",
      );
      assert.equal(inspector.status, 0);
      save(artifacts, "inspector.json", JSON.parse(inspector.stdout));
      inspectorPackets++;
      for (const [name, value] of Object.entries(inputs)) {
        if (!name.endsWith(".vue")) continue;
        const source = text(value);
        const parsed: unknown = (
          object(native).parseSfc as (source: string, options: unknown) => unknown
        )(source, { filename: name });
        const analyzed: unknown = (
          object(native).analyzeSfc as (source: string, options: unknown) => unknown
        )(source, { filename: name, profile: false, cache: false });
        save(artifacts, path.basename(name) + "-public-native.json", {
          sourceName: name,
          sourceSha256: sha(source),
          parsed,
          analyzed,
        });
        publicNativePackets++;
      }
      let runtime: unknown = null;
      if (kind === "composable") {
        runtime = probeRuntime(row, inputs, artifacts);
        runtimeProbes++;
      }
      rows.push({
        id,
        kind,
        sourcePins: inputPins,
        doctor: doctorPacket,
        lint: lintPacket,
        statuses: [doctor.status, lint.status],
        runtime,
      });
      save(out, "complete-before-rows.json", rows);
    } catch (error) {
      failures.push({ id, error: String(error) });
      save(out, "observer-failures.json", failures);
    }
  }
}
const result = {
  stage: "before-only",
  productFixed: false,
  issue: 10914,
  observedVersion: version,
  candidate,
  observerHead: process.env.GITHUB_SHA,
  corpusSha256: sha(raw),
  casesObserved: rows.length,
  cliPackets,
  inspectorPackets,
  publicNativePackets,
  runtimeProbes,
  failures,
};
save(out, "before-observer-result.json", result);
assert.deepEqual(failures, []);
assert.equal(rows.length, 41);
assert.equal(cliPackets, 82);
assert.equal(inspectorPackets, 41);
assert.equal(publicNativePackets, 82);
assert.equal(runtimeProbes, 14);
console.log(JSON.stringify(result));
