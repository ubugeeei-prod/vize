import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import {
  mkdtemp,
  mkdir,
  readFile,
  readdir,
  realpath,
  rm,
  stat,
  symlink,
  utimes,
  writeFile,
} from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { basename, dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";

const require = createRequire(import.meta.url);
const vite = dirname(require.resolve("vite/package.json"));
const loader = new URL("../../crates/vize_carton/src/config/loader/js.rs", import.meta.url);
const rust = await readFile(loader, "utf8");
const embedded =
  (
    await Promise.all(
      [...rust.matchAll(/include_str!\("([^"]+)"\)/g)].map((match) =>
        readFile(new URL(match[1], loader), "utf8"),
      ),
    )
  ).join("") + [...rust.matchAll(/r#"([\s\S]*?)"#/g)][0][1];
const cacheVariables = [
  "NODE_COMPILE_CACHE",
  "NODE_DISABLE_COMPILE_CACHE",
  "NODE_COMPILE_CACHE_PORTABLE",
  "NODE_V8_COVERAGE",
];

async function fixture(run: (root: string, cache: string) => Promise<void>) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-compile-cache-test-")));
  try {
    await mkdir(join(root, "node_modules"));
    await symlink(vite, join(root, "node_modules/vite"), "junction");
    await writeFile(join(root, "package.json"), '{"type":"module"}');
    const cache = join(root, "compiled");
    await mkdir(cache, { mode: 0o700 });
    await run(root, cache);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function observed(selected: string, cache?: string, settings: NodeJS.ProcessEnv = {}) {
  const env: NodeJS.ProcessEnv = { ...process.env, VIZE_TRACE_EDITOR_PREPARATION: "0" };
  for (const name of cacheVariables) delete env[name];
  delete env.NODE_OPTIONS;
  Object.assign(env, settings);
  const args = ["--input-type=module", "-e", embedded, selected];
  if (cache) args.push(cache);
  args.push("--observe-config-inputs");
  const child = spawn(process.execPath, args, {
    cwd: dirname(selected),
    env,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let stderr = "";
  child.stderr.setEncoding("utf8").on("data", (part) => {
    stderr += part;
  });
  const exited = new Promise<number | null>((resolve, reject) => {
    child.on("error", reject);
    child.on("close", resolve);
  });
  const output = child.stdout[Symbol.asyncIterator]();
  let buffered = Buffer.alloc(0);
  const acknowledgements: string[][] = [];
  async function fill(length: number) {
    while (buffered.length < length) {
      const next = await output.next();
      assert.equal(next.done, false, `unexpected child EOF: ${stderr}`);
      buffered = Buffer.concat([buffered, next.value]);
    }
  }
  try {
    for (;;) {
      while (!buffered.includes(10)) await fill(buffered.length + 1);
      const newline = buffered.indexOf(10);
      const header = buffered.subarray(0, newline).toString();
      buffered = buffered.subarray(newline + 1);
      if (header === "R") break;
      assert.match(header, /^B\d+$/);
      const length = Number(header.slice(1));
      assert.ok(length <= 65536);
      await fill(length);
      acknowledgements.push(JSON.parse(buffered.subarray(0, length).toString()));
      buffered = buffered.subarray(length);
      child.stdin.write(Buffer.from([6]));
    }
    child.stdin.end();
    for (;;) {
      const next = await output.next();
      if (next.done) break;
      buffered = Buffer.concat([buffered, next.value]);
    }
    return {
      pid: child.pid,
      status: await exited,
      stderr,
      acknowledgements,
      value: JSON.parse(buffered.toString()),
    };
  } finally {
    if (child.exitCode === null) child.kill();
    await exited;
  }
}

async function packageInputs(root: string) {
  const directory = join(root, "node_modules/cache-probe");
  await mkdir(directory);
  await writeFile(
    join(directory, "package.json"),
    '{"name":"cache-probe","type":"module","exports":{"import":"./esm.mjs","require":"./cjs.cjs"}}',
  );
  const esm = join(directory, "esm.mjs");
  const cjs = join(directory, "cjs.cjs");
  await writeFile(
    esm,
    "globalThis.esmLoads=(globalThis.esmLoads??0)+1; export default {value:91,loads:globalThis.esmLoads};",
  );
  await writeFile(
    cjs,
    "globalThis.cjsLoads=(globalThis.cjsLoads??0)+1; module.exports={value:1,loads:globalThis.cjsLoads};",
  );
  const timestamp = new Date("2020-01-02T03:04:05.000Z");
  await utimes(esm, timestamp, timestamp);
  await utimes(cjs, timestamp, timestamp);
  return { esm, cjs };
}

function source(counter: string) {
  return `import esm from 'cache-probe'; import {createRequire} from 'node:module';
    import {appendFileSync} from 'node:fs'; const cjs=createRequire(import.meta.url)('cache-probe');
    if(esm.loads!==1 || cjs.loads!==1) throw new Error('evaluated namespace was reused');
    export default () => {appendFileSync(${JSON.stringify(counter)},'once\\n');
      return {vue:{formatter:{printWidth:esm.value+cjs.value}}};};`;
}

function sameResult(
  left: Awaited<ReturnType<typeof observed>>,
  right: Awaited<ReturnType<typeof observed>>,
) {
  assert.notEqual(left.pid, right.pid);
  assert.deepEqual(
    {
      status: left.status,
      stderr: left.stderr,
      acknowledgements: left.acknowledgements,
      value: left.value,
    },
    {
      status: right.status,
      stderr: right.stderr,
      acknowledgements: right.acknowledgements,
      value: right.value,
    },
  );
}

async function cacheFiles(cache: string): Promise<string[]> {
  return (await readdir(cache, { recursive: true, withFileTypes: true }))
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name));
}

test("compiled artifacts preserve fresh child globals, whole values, dependencies, ACKs and one factory per child", async () => {
  await fixture(async (root, cache) => {
    await packageInputs(root);
    const selected = join(root, "vite.config.mjs");
    const counter = join(root, "factories.txt");
    await writeFile(selected, source(counter));
    const original = await observed(selected);
    assert.equal(original.status, 0, original.stderr);
    assert.equal(original.value.config.formatter.printWidth, 92);
    const first = await observed(selected, cache);
    const second = await observed(selected, cache);
    sameResult(original, first);
    sameResult(first, second);
    const populated = await cacheFiles(cache);
    assert.ok(populated.length > 0, "actual Node exit flushed compiled artifacts");
    for (const path of populated) await writeFile(path, "not a valid Node compiled artifact");
    const repairedCache = await observed(selected, cache);
    sameResult(original, repairedCache);
    assert.equal(await readFile(counter, "utf8"), "once\nonce\nonce\nonce\n");
  });
});

test("cached CJS and ESM obey same-size restored-mtime content changes and a replaced lexical symlink", async () => {
  await fixture(async (root, cache) => {
    const { esm, cjs } = await packageInputs(root);
    const selected = join(root, "vite.config.mjs");
    const counter = join(root, "factories.txt");
    await writeFile(selected, source(counter));
    const first = await observed(selected, cache);
    assert.equal(first.status, 0, first.stderr);
    assert.equal(first.value.config.formatter.printWidth, 92);
    for (const [path, before, after] of [
      [esm, "value:91", "value:93"],
      [cjs, "value:1", "value:3"],
    ]) {
      const metadata = await stat(path);
      const original = await readFile(path, "utf8");
      const replacement = original.replace(before, after);
      assert.notEqual(replacement, original);
      assert.equal(Buffer.byteLength(replacement), metadata.size);
      await writeFile(path, replacement);
      await utimes(path, metadata.atime, metadata.mtime);
      assert.equal((await stat(path)).mtimeMs, metadata.mtimeMs);
    }
    const changed = await observed(selected, cache);
    assert.equal(changed.status, 0, changed.stderr);
    assert.equal(changed.value.config.formatter.printWidth, 96);
    sameResult(changed, await observed(selected));
    const moved = join(root, "replacement.mjs");
    await writeFile(
      moved,
      "globalThis.esmLoads=(globalThis.esmLoads??0)+1; export default {value:95,loads:globalThis.esmLoads};",
    );
    await rm(esm);
    await symlink(moved, esm);
    const retargeted = await observed(selected, cache);
    assert.equal(retargeted.status, 0, retargeted.stderr);
    assert.equal(retargeted.value.config.formatter.printWidth, 98);
    sameResult(retargeted, await observed(selected));
    assert.equal(await readFile(counter, "utf8"), "once\nonce\nonce\nonce\nonce\n");
  });
});

test("two app roots share only compiled code while selected cwd, environment and factories stay independent", async () => {
  await fixture(async (root, cache) => {
    const counter = join(root, "factories.txt");
    const pids = new Set<number | undefined>();
    for (const [name, width] of [
      ["app-a", 91],
      ["app-b", 97],
    ] as const) {
      const app = join(root, name);
      await mkdir(app);
      const selected = join(app, "vite.config.mjs");
      await writeFile(
        selected,
        `import {appendFileSync} from 'node:fs';
        globalThis.factoryLoads=(globalThis.factoryLoads??0)+1;
        if(globalThis.factoryLoads!==1 || process.cwd()!==${JSON.stringify(app)}) throw new Error('app context leaked');
        export default()=>{appendFileSync(${JSON.stringify(counter)},'once\\n');
        return {vue:{formatter:{printWidth:Number(process.env.AUTHORED_WIDTH)}}};};`,
      );
      const loaded = await observed(selected, cache, { AUTHORED_WIDTH: String(width) });
      assert.equal(loaded.status, 0, loaded.stderr);
      assert.equal(loaded.value.config.formatter.printWidth, width);
      assert.equal(pids.has(loaded.pid), false);
      pids.add(loaded.pid);
    }
    assert.equal(await readFile(counter, "utf8"), "once\nonce\n");
  });
});

test("authored whole failure is unchanged and missing helpers repair without a retained evaluated namespace", async () => {
  await fixture(async (root, cache) => {
    const selected = join(root, "vite.config.mjs");
    const failedCounter = join(root, "failed-factories.txt");
    await writeFile(
      selected,
      `import {appendFileSync} from 'node:fs'; export default()=>{
        appendFileSync(${JSON.stringify(failedCounter)},'once\\n');
        const error=new Error('authored-cache-failure');
        error.stack='Error: authored-cache-failure\\n    at authored-config:1:1'; throw error;};`,
    );
    const original = await observed(selected);
    const cached = await observed(selected, cache);
    assert.notEqual(original.status, 0);
    assert.match(original.stderr, /authored-cache-failure/);
    const identity = (result: Awaited<ReturnType<typeof observed>>) => {
      const raw = JSON.stringify(result);
      const { failureArtifacts: mappings, ...value } = result.value;
      assert.equal(mappings.length, 3);
      const adapter = mappings[0][0] as string;
      assert.equal(basename(dirname(adapter)).startsWith("vize-config-module-"), true);
      assert.deepEqual(mappings, [
        [adapter, selected],
        [pathToFileURL(adapter).href, pathToFileURL(selected).href],
        [relative(root, adapter), relative(root, selected)],
      ]);
      const stderr = [...mappings]
        .sort(([a], [b]) => b.length - a.length)
        .reduce((text, [owned, authored]) => text.split(owned).join(authored), result.stderr);
      assert.equal(JSON.stringify(result), raw, "raw diagnostics and packets stay untouched");
      return { status: result.status, acknowledgements: result.acknowledgements, value, stderr };
    };
    assert.notEqual(original.pid, cached.pid);
    assert.notDeepEqual(original.value.failureArtifacts, cached.value.failureArtifacts);
    assert.deepEqual(identity(original), identity(cached));
    const changedError = {
      ...identity(cached),
      stderr: identity(cached).stderr.replaceAll(
        "authored-cache-failure",
        "different-authored-error",
      ),
    };
    assert.throws(() => assert.deepEqual(identity(original), changedError), {
      code: "ERR_ASSERTION",
    });
    assert.equal(await readFile(failedCounter, "utf8"), "once\nonce\n");
    await packageInputs(root);
    const counter = join(root, "factories.txt");
    await writeFile(selected, source(counter));
    const esm = join(root, "node_modules/cache-probe/esm.mjs");
    const bytes = await readFile(esm);
    await rm(esm);
    assert.notEqual((await observed(selected, cache)).status, 0);
    await assert.rejects(readFile(counter), { code: "ENOENT" });
    await writeFile(esm, bytes);
    const repaired = await observed(selected, cache);
    assert.equal(repaired.status, 0, repaired.stderr);
    assert.equal(repaired.value.config.formatter.printWidth, 92);
    sameResult(repaired, await observed(selected));
    assert.equal(await readFile(counter, "utf8"), "once\nonce\n");
  });
});

test("explicit disable, coverage and external cache policy leave the owned directory unused", async () => {
  await fixture(async (root, cache) => {
    const selected = join(root, "vite.config.mjs");
    await writeFile(selected, "export default {vue:{formatter:{printWidth:93}}};");
    for (const settings of [
      { NODE_DISABLE_COMPILE_CACHE: "1" },
      { NODE_V8_COVERAGE: join(root, "coverage") },
      { NODE_COMPILE_CACHE: join(root, "external") },
      { NODE_COMPILE_CACHE_PORTABLE: "1" },
    ]) {
      const original = await observed(selected, undefined, settings);
      const owned = await observed(selected, cache, settings);
      assert.equal(owned.status, 0, owned.stderr);
      sameResult(original, owned);
      assert.deepEqual(await cacheFiles(cache), []);
    }
    const unavailable = join(root, "not-a-directory");
    await writeFile(unavailable, "owned compile cache unavailable");
    const fallback = await observed(selected, unavailable);
    assert.equal(fallback.status, 0, fallback.stderr);
    sameResult(fallback, await observed(selected));
  });
});
