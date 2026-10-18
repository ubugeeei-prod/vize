import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { runInNewContext } from "node:vm";

import {
  cleanupProjectNative,
  stageProjectNative,
} from "../../npm/oxlint/scripts/project-native-staging.ts";

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

void test("the actual caller stages standalone and emits its exact owned path only when GitHub output exists", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-caller-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const source = path.join(directory, "current.node");
  const sourceReceipt = path.join(directory, "receipt.json");
  const bytes = Buffer.from("current standalone source native binary\0");
  fs.writeFileSync(source, bytes);
  fs.writeFileSync(sourceReceipt, '{"source":"standalone caller"}\n');
  const caller = fs.readFileSync(
    fileURLToPath(new URL("../../npm/oxlint/scripts/check-project-transport.mjs", import.meta.url)),
    "utf8",
  );
  const start = caller.indexOf("const staged = stageProjectNative(");
  const end = caller.indexOf("const [mixedDirectory8507, n8nResults] =", start);
  assert.ok(start >= 0 && end > start, "the complete live staging paragraph must be present");
  const context = {
    artifacts: undefined,
    stageProjectNative,
    receipt: { frozen: { path: source, sha256: sha256(bytes) } },
    nativeHistoryReceipt: () => sourceReceipt,
    path,
    root: directory,
    nativeDir: directory,
    fs,
  };
  runInNewContext(caller.slice(start, end), { ...context, process: { env: {} } });
  const artifacts = path.join(directory, "target/oxlint-original-project-transport");
  const standalone = fs.readdirSync(artifacts);
  assert.equal(standalone.length, 1);
  assert.deepEqual(
    fs.readFileSync(path.join(artifacts, standalone[0]!, "source-native.node")),
    bytes,
  );

  const output = path.join(directory, "github-output");
  runInNewContext(caller.slice(start, end), {
    ...context,
    process: { env: { GITHUB_OUTPUT: output } },
  });
  const staged = fs.readdirSync(artifacts).filter((name) => name !== standalone[0]);
  assert.equal(staged.length, 1);
  assert.equal(
    fs.readFileSync(output, "utf8"),
    `staging-directory=${path.join(artifacts, staged[0]!)}\nphase-walltime=${path.join(artifacts, staged[0]!, "phase-walltime.json")}\n`,
  );
  assert.deepEqual(fs.readFileSync(path.join(artifacts, staged[0]!, "source-native.node")), bytes);
});

void test("cached fixed-target staging fails exclusively, while consecutive owned stages preserve each source", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "target/oxlint-original-project-transport");
  fs.mkdirSync(artifacts, { recursive: true });
  const stale = path.join(artifacts, "source-native.node");
  const staleBytes = Buffer.from("previous cached source native binary\0");
  fs.writeFileSync(stale, staleBytes);
  const priorEvidence = path.join(artifacts, "qualification.json");
  const priorBytes = Buffer.from('{"source":"previous whole qualification"}\n');
  fs.writeFileSync(priorEvidence, priorBytes);
  const source = path.join(directory, "current.node");
  const receipt = path.join(directory, "receipt.json");
  const firstBytes = Buffer.from("first current native binary\0");
  const firstReceipt = Buffer.from('{"source":"first exact source"}\n');
  fs.writeFileSync(source, firstBytes);
  fs.writeFileSync(receipt, firstReceipt);

  // The original live call at signed main9d fails before either real host starts.
  assert.throws(() => fs.copyFileSync(source, stale, fs.constants.COPYFILE_EXCL), {
    code: "EEXIST",
  });
  const first = stageProjectNative(artifacts, source, receipt, sha256(firstBytes));
  const secondBytes = Buffer.from("second different current native binary\0");
  const secondReceipt = Buffer.from('{"source":"second exact source"}\n');
  fs.writeFileSync(source, secondBytes);
  fs.writeFileSync(receipt, secondReceipt);
  const second = stageProjectNative(artifacts, source, receipt, sha256(secondBytes));

  assert.notEqual(first.artifacts, second.artifacts);
  assert.notEqual(first.binary, second.binary);
  for (const stage of [first, second]) {
    assert.equal(path.dirname(stage.artifacts), artifacts);
    assert.match(path.basename(stage.artifacts), /^run-/u);
    assert.equal(stage.binary, path.join(stage.artifacts, "source-native.node"));
  }
  assert.deepEqual(fs.readFileSync(first.binary), firstBytes);
  assert.deepEqual(fs.readFileSync(second.binary), secondBytes);
  assert.deepEqual(fs.readFileSync(path.join(first.artifacts, "build-receipt.json")), firstReceipt);
  assert.deepEqual(
    fs.readFileSync(path.join(second.artifacts, "build-receipt.json")),
    secondReceipt,
  );
  assert.deepEqual(fs.readFileSync(stale), staleBytes);
  assert.deepEqual(fs.readFileSync(priorEvidence), priorBytes);
  assert.throws(() => fs.copyFileSync(source, first.binary, fs.constants.COPYFILE_EXCL), {
    code: "EEXIST",
  });
  assert.deepEqual(fs.readFileSync(first.binary), firstBytes);
  assert.throws(() => cleanupProjectNative(artifacts, first.artifacts), { code: "ENOENT" });
  fs.writeFileSync(path.join(first.artifacts, "qualification.json"), '{"complete":true}\n');
  cleanupProjectNative(artifacts, first.artifacts);
  assert.equal(fs.existsSync(first.artifacts), false);
  assert.deepEqual(fs.readFileSync(second.binary), secondBytes);
  assert.deepEqual(fs.readFileSync(stale), staleBytes);
  assert.deepEqual(fs.readFileSync(priorEvidence), priorBytes);
});

void test("wrong current source hash is rejected and preserves only its own partial evidence", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-refusal-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "transport");
  fs.mkdirSync(artifacts);
  const untouched = path.join(artifacts, "unowned-cached-run");
  fs.mkdirSync(untouched);
  fs.writeFileSync(path.join(untouched, "source-native.node"), "unowned previous source");
  const source = path.join(directory, "current.node");
  const receipt = path.join(directory, "receipt.json");
  const bytes = Buffer.from("current source fails expected binary hash\0");
  fs.writeFileSync(source, bytes);
  fs.writeFileSync(receipt, '{"source":"must not qualify"}\n');

  assert.throws(() => stageProjectNative(artifacts, source, receipt, "0".repeat(64)), {
    code: "ERR_ASSERTION",
  });
  const owned = fs.readdirSync(artifacts).filter((name) => name.startsWith("run-"));
  assert.equal(owned.length, 1);
  assert.deepEqual(fs.readFileSync(path.join(artifacts, owned[0]!, "source-native.node")), bytes);
  assert.equal(fs.existsSync(path.join(artifacts, owned[0]!, "build-receipt.json")), false);
  assert.equal(
    fs.readFileSync(path.join(untouched, "source-native.node"), "utf8"),
    "unowned previous source",
  );
  assert.throws(() => cleanupProjectNative(artifacts, artifacts), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, untouched), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, path.join(artifacts, owned[0]!)), {
    code: "ENOENT",
  });
  assert.deepEqual(fs.readFileSync(path.join(artifacts, owned[0]!, "source-native.node")), bytes);
});

void test("cleanup refuses an outside directory and a symlink to an unowned completed stage", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-cleanup-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "transport");
  fs.mkdirSync(artifacts);
  const outside = fs.mkdtempSync(path.join(directory, "run-"));
  fs.writeFileSync(path.join(outside, "qualification.json"), '{"complete":true}\n');
  const link = path.join(artifacts, "run-123456");
  fs.symlinkSync(outside, link, "dir");
  assert.throws(() => cleanupProjectNative(artifacts, outside), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, link), { code: "ERR_ASSERTION" });
  assert.equal(
    fs.readFileSync(path.join(outside, "qualification.json"), "utf8"),
    '{"complete":true}\n',
  );
  assert.equal(fs.lstatSync(link).isSymbolicLink(), true);
});

// Authored CommonJS modules exercise real Node preload/cache/environment behavior.
// These are not native addons and cannot confer actual Oxlint/NAPI acceptance.
function sourcePreloadFixture(t: { after: (cleanup: () => void) => void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-preload-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const helper = path.join(root, "npm/oxlint/scripts/project-native-staging.ts");
  const wrapper = path.join(root, "npm/native/index.js");
  fs.mkdirSync(path.dirname(helper), { recursive: true });
  fs.mkdirSync(path.dirname(wrapper));
  fs.copyFileSync(
    fileURLToPath(new URL("../../npm/oxlint/scripts/project-native-staging.ts", import.meta.url)),
    helper,
  );
  const binary = path.join(root, "authored-image.cjs");
  fs.writeFileSync(binary, "module.exports={authoredImage:true};\n");
  fs.writeFileSync(
    wrapper,
    `globalThis.wrapperLoads=(globalThis.wrapperLoads??0)+1;
module.exports=require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
`,
  );
  const identify = (file: string) => ({ path: file, sha256: sha256(fs.readFileSync(file)) });
  return {
    root,
    helper,
    wrapper,
    binary,
    identify,
    custody: {
      schema: "vize.oxlint.original-sfc-preload",
      wrapper: identify(wrapper),
      binary: identify(binary),
    },
  };
}

function runPreloadControl(fixture: ReturnType<typeof sourcePreloadFixture>, body: string) {
  const script = path.join(fixture.root, "control.mjs");
  fs.writeFileSync(
    script,
    `import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {preloadProjectNative} from ${JSON.stringify(pathToFileURL(fixture.helper).href)};
const require=createRequire(import.meta.url),custody=${JSON.stringify(fixture.custody)};
${body}
process.stdout.write(JSON.stringify({override:process.env.NAPI_RS_NATIVE_LIBRARY_PATH??null,loads:globalThis.wrapperLoads??0}));
`,
  );
  const env = { ...process.env };
  for (const key of [
    "NODE_OPTIONS",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_OXLINT_NATIVE_CUSTODY",
  ])
    delete env[key];
  const result = spawnSync(process.execPath, [script], { env, encoding: "utf8", timeout: 5000 });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.equal(result.stderr, "");
  return JSON.parse(result.stdout) as { override: null; loads: number };
}

void test("source preload loads only its exact wrapper/image once and leaves host and child overrides absent", (t) => {
  const fixture = sourcePreloadFixture(t);
  assert.deepEqual(
    runPreloadControl(
      fixture,
      `
preloadProjectNative(custody);
const first=require(custody.wrapper.path),second=require(custody.wrapper.path);
assert.equal(first,second);
assert.equal(first,require.cache[custody.binary.path].exports);
assert.deepEqual(first,{authoredImage:true});
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH,undefined);
const {spawnSync}=require('node:child_process');
const host=spawnSync(process.execPath,['-e',"process.stdout.write(JSON.stringify(process.env.NAPI_RS_NATIVE_LIBRARY_PATH??null))"],{encoding:'utf8',timeout:3000});
assert.equal(host.status,0);assert.equal(host.stdout,'null');assert.equal(host.stderr,'');
`,
    ),
    { override: null, loads: 1 },
  );
});

void test("throwing wrapper preserves the exact error and restores environment before caller recovery", (t) => {
  const fixture = sourcePreloadFixture(t);
  fs.writeFileSync(
    fixture.wrapper,
    "globalThis.wrapperLoads=1;throw new Error('whole authored wrapper failure');\n",
  );
  fixture.custody.wrapper = fixture.identify(fixture.wrapper);
  assert.deepEqual(
    runPreloadControl(
      fixture,
      `
assert.throws(()=>preloadProjectNative(custody),{name:'Error',message:'whole authored wrapper failure'});
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH,undefined);
assert.equal(require.cache[custody.wrapper.path],undefined);
`,
    ),
    { override: null, loads: 1 },
  );
});

for (const control of [
  "wrong-wrapper",
  "wrapper-bytes",
  "image-bytes",
  "image-path",
  "missing-image-load",
  "wrong-image-export",
] as const) {
  void test(`source preload refuses ${control} without leaving a host override`, (t) => {
    const fixture = sourcePreloadFixture(t);
    let expectedLoads = 0;
    if (control === "wrong-wrapper") {
      const other = path.join(fixture.root, "other-wrapper.js");
      fs.copyFileSync(fixture.wrapper, other);
      fixture.custody.wrapper = fixture.identify(other);
    } else if (control === "wrapper-bytes")
      fs.appendFileSync(fixture.wrapper, "// altered wrapper\n");
    else if (control === "image-bytes") fs.appendFileSync(fixture.binary, "// altered image\n");
    else if (control === "image-path") {
      const link = path.join(fixture.root, "image-link.cjs");
      fs.symlinkSync(fixture.binary, link);
      fixture.custody.binary.path = link;
    } else {
      fs.writeFileSync(
        fixture.wrapper,
        "globalThis.wrapperLoads=1;" +
          (control === "wrong-image-export"
            ? "require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);"
            : "") +
          "module.exports={unrelated:true};\n",
      );
      fixture.custody.wrapper = fixture.identify(fixture.wrapper);
      expectedLoads = 1;
    }
    assert.deepEqual(
      runPreloadControl(
        fixture,
        `
assert.throws(()=>preloadProjectNative(custody),{code:'ERR_ASSERTION'});
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH,undefined);
assert.equal(require.cache[custody.binary.path]!==undefined,${control === "wrong-image-export"});
`,
      ),
      { override: null, loads: expectedLoads },
    );
  });
}
