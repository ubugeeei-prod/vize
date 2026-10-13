import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const history = Object.freeze([
  "b45d789b317933d48dadbba4841c8e353231547b",
  "6fb01f335952780d7a6321ca092049753645c83d",
  "ea1fc59ee6ef61a3509b1ef42a6fa7aacd9f8586",
  "e02a861cc1c6ad12724036c70bfdd5d48173b189",
  "f6017bf3a9837136f7ce60d52d67ecc3ee649dd0",
  "e78d2cb93e278032cd262201b0a444273618ee25",
  "3a680d0a0b59b720af0d23a70a85cf56c5c4f175",
  "a93e78147b0990c813e965a7868b32dda341eb77",
  "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  "49d95b8e93a9f5bcb73e0c1c307815d4b3a9d570",
  "5c54b1195fa8c0bf151e0a2862bf4238a3597b4e",
  "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f",
  "2489128d5f8fab1b7442a1e5c09b4e1701cbf185",
  "3dda8f9d71fbd7e701f0805187daff6191865190",
  "66d2cfab35dd6dc8f730371b3b127543b6cc9505",
  "f518d884e3a1c70ce147fb966093cd9721e0e81f",
  "8a5f5b8f3a39e123ad20ad8ccd9ecd3b39276695",
  "d95c151c18d85c8fdd95541291bc614e44790101",
  "5c8f3825fd2eb698c27d3b97d0a47b0671b33e01",
  "43afa9b09ae0f334d7fcba41f721ac52a8d1ab23",
]);

const sources = new Set([
  "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38",
  "19b451f951137814a8a8d36b57d200a23100344e",
  "2da71113a3f8ab528b1a9cda20fc6a4c2bd273f6",
  "ebbdc1b1e1559d7f4efd55978bd8b52f127acb3a",
]);
function observePrime(event) {
  const directory = process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR;
  if (!directory) return;
  fs.mkdirSync(directory, { recursive: true });
  fs.appendFileSync(
    path.join(directory, "history-prime-" + process.pid + ".jsonl"),
    JSON.stringify({ time: new Date().toISOString(), pid: process.pid, ...event }) + "\n",
  );
}
function alive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}
function stopGroup(pid) {
  if (!pid) return;
  observePrime({ event: "cleanup-signal", group: pid, signal: "SIGKILL" });
  try {
    process.kill(process.platform === "win32" ? pid : -pid, "SIGKILL");
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}
/** One finite public history set; serialize strict hosted fetches across source entrypoints. */
export function primeStarCounterpartHistory(owner, { timeoutMs = 240_000 } = {}) {
  assert(Number.isSafeInteger(timeoutMs) && timeoutMs > 0 && timeoutMs <= 240_000);
  const root = fs.realpathSync(owner),
    required = [...sources, ...history];
  const present = () => {
    const records = git(
      root,
      ["cat-file", "--batch-check=%(objectname) %(objecttype)"],
      required.join("\n") + "\n",
    )
      .toString()
      .trim()
      .split("\n");
    assert.equal(records.length, required.length);
    return records.every((record, index) => record === required[index] + " commit");
  };
  if (present()) return;
  observePrime({ event: "missing-history", root, required, timeoutMs });
  const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
  const directory = fs.realpathSync(path.resolve(root, common, "objects/info"));
  const lock = path.join(directory, "star-counterpart-prime.lock");
  const token = process.pid + ":" + Date.now(),
    workerFile = lock + ".worker." + token,
    deadline = performance.now() + timeoutMs;
  let acquired;
  try {
    while (!acquired) {
      try {
        const descriptor = fs.openSync(lock, "wx", 0o600);
        acquired = fs.fstatSync(descriptor);
        try {
          fs.writeFileSync(descriptor, token);
        } finally {
          fs.closeSync(descriptor);
        }
      } catch (error) {
        if (error.code !== "EEXIST") throw error;
        assert(performance.now() < deadline, "bounded owned history priming wait");
        if (present()) return;
        try {
          const stat = fs.lstatSync(lock),
            previous = fs.readFileSync(lock, "utf8");
          assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o600);
          const parsed = /^([1-9][0-9]*):([0-9]+)$/u.exec(previous);
          if (parsed && !alive(Number(parsed[1]))) {
            let finished = true;
            const previousWorkerFile = lock + ".worker." + previous;
            if (fs.existsSync(previousWorkerFile)) {
              const worker = fs.readFileSync(previousWorkerFile, "utf8");
              assert(worker.startsWith(previous + ":"), "same dead owner's worker");
              const pid = Number(worker.slice(previous.length + 1));
              assert(Number.isSafeInteger(pid) && pid > 0);
              finished = !alive(process.platform === "win32" ? pid : -pid);
              if (finished) fs.unlinkSync(previousWorkerFile);
            }
            if (finished) {
              const current = fs.lstatSync(lock);
              if (
                current.ino === stat.ino &&
                current.dev === stat.dev &&
                fs.readFileSync(lock, "utf8") === previous
              )
                fs.unlinkSync(lock);
              continue;
            }
          }
        } catch (observationError) {
          if (observationError.code !== "ENOENT") throw observationError;
          continue;
        }
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
      }
    }
    const remaining = Math.ceil(deadline - performance.now()) - 500;
    assert(remaining > 0, "remaining bounded history priming budget");
    const helper = new URL(
      "../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts",
      import.meta.url,
    ).href;
    const observation = `const observe=${observePrime.toString()};`;
    const monitor = `const fs=require('node:fs'), path=require('node:path'); ${observation} const owner=${process.pid}; const group=Number(process.argv[1]); const end=Date.now()+Number(process.argv[2]); observe({event:'monitor-start',owner,group,end}); const alive=p=>{try{process.kill(p,0);return true}catch(e){if(e.code==='ESRCH')return false;throw e}}; process.on('SIGTERM',()=>{}); const timer=setInterval(()=>{const ownerAlive=alive(owner),groupAlive=alive(group),expired=Date.now()>=end;if(!ownerAlive||!groupAlive||expired){clearInterval(timer);observe({event:'monitor-signal',owner,group,ownerAlive,groupAlive,expired,signal:'SIGTERM'});try{process.kill(-group,'SIGTERM')}catch(e){if(e.code!=='ESRCH')throw e}setTimeout(()=>{observe({event:'monitor-signal',owner,group,signal:'SIGKILL'});try{process.kill(-group,'SIGKILL')}catch(e){if(e.code!=='ESRCH')throw e}},200)}},50);`;
    const script = `import fs from 'node:fs'; import path from 'node:path'; import {spawn} from 'node:child_process'; import {ensureHostedLiteralSource} from ${JSON.stringify(helper)}; ${observation} const lock=${JSON.stringify(lock)}, token=${JSON.stringify(token)}; observe({event:'worker-start',owner:${process.pid},group:process.pid}); if(fs.readFileSync(lock,'utf8')!==token) throw Error('same owned lock required'); fs.writeFileSync(${JSON.stringify(workerFile)},token+':'+process.pid,{flag:'wx',mode:0o600}); if(fs.readFileSync(lock,'utf8')!==token) throw Error('same live owner lock required'); try{process.kill(${process.pid},0)}catch(e){throw Error('live owner required')} const monitor=spawn(process.execPath,['-e',${JSON.stringify(monitor)},String(process.pid),${JSON.stringify(String(remaining))}],{stdio:'ignore'}); observe({event:'monitor-launched',monitor:monitor.pid,owner:${process.pid},group:process.pid}); monitor.unref(); for(const source of ${JSON.stringify(required)}) ensureHostedLiteralSource(${JSON.stringify(root)},source); observe({event:'worker-complete',owner:${process.pid},group:process.pid});`;
    let result;
    try {
      result = spawnSync(process.execPath, ["--input-type=module", "-e", script], {
        cwd: root,
        env: process.env,
        stdio: "inherit",
        detached: process.platform !== "win32",
        timeout: remaining + 500,
        killSignal: "SIGKILL",
      });
      observePrime({
        event: "worker-result",
        group: result.pid,
        status: result.status,
        signal: result.signal,
        error: result.error ? { code: result.error.code, message: result.error.message } : null,
      });
      if (result.error) throw result.error;
      assert.equal(result.status, 0, "bounded strict literal history worker");
    } finally {
      stopGroup(result?.pid);
    }
    assert(present(), "complete finite source history before counterpart provisioning");
  } finally {
    if (acquired) {
      const current = fs.lstatSync(lock);
      assert(
        current.ino === acquired.ino && current.dev === acquired.dev,
        "same owned priming lock inode",
      );
      if (fs.existsSync(workerFile)) {
        assert(
          fs.readFileSync(workerFile, "utf8").startsWith(token + ":"),
          "same owned history worker",
        );
        fs.unlinkSync(workerFile);
      }
      fs.unlinkSync(lock);
    }
  }
}

const object = (kind, body) =>
  createHash("sha1")
    .update(kind + " " + body.length + "\0")
    .update(body)
    .digest("hex");
const safeName = (name) =>
  typeof name === "string" &&
  /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u.test(name) &&
  !name.split("/").some((part) => [".", "..", ".git"].includes(part.toLowerCase()));

function selectedNames(names) {
  assert(Array.isArray(names) && names.length, "nonempty selected counterpart names");
  assert(names.every(safeName), "safe relative counterpart names only");
  assert.equal(new Set(names).size, names.length, "unique selected counterpart names");
  return [...names];
}

function git(root, args, input) {
  const env = { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" };
  for (const key of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_INDEX_FILE",
    "GIT_NAMESPACE",
    "GIT_SHALLOW_FILE",
  ])
    delete env[key];
  return execFileSync(
    "git",
    [
      "--no-replace-objects",
      "--literal-pathspecs",
      "-c",
      "gc.auto=0",
      "-c",
      "maintenance.auto=false",
      ...args,
    ],
    { cwd: root, input, env, maxBuffer: 64 * 1024 * 1024 },
  );
}

function context(owner, source) {
  assert(sources.has(source), "only the four literal Star/SDK source counterparts");
  const root = fs.realpathSync(owner);
  assert.equal(
    fs.realpathSync(git(root, ["rev-parse", "--show-toplevel"]).toString().trim()),
    root,
    "counterpart owner is the repository root",
  );
  const commit = git(root, ["cat-file", "commit", source]);
  assert.equal(object("commit", commit), source, "whole literal counterpart commit");
  const tree = /^tree ([a-f0-9]{40})\n/u.exec(commit.toString());
  assert(tree, "literal counterpart commit tree");
  return { root, source, tree: tree[1] };
}

function entries(owned, names = []) {
  const bytes = git(owned.root, [
    "ls-tree",
    "-r",
    "-z",
    "--full-tree",
    owned.source,
    ...(names.length ? ["--", ...names] : []),
  ]);
  assert.deepEqual(Buffer.from(bytes.toString()), bytes, "whole raw counterpart tree bytes");
  const tree = new Map();
  if (!bytes.length) return tree;
  assert.equal(bytes.at(-1), 0, "complete NUL-framed counterpart tree");
  const rows = bytes.toString().split("\0");
  assert.equal(rows.pop(), "");
  for (const row of rows) {
    const match = /^(100644|100755|120000|160000) (blob|commit) ([a-f0-9]{40})\t(.+)$/u.exec(row);
    assert(match, "complete raw counterpart tree row");
    const name = match[4];
    assert(!tree.has(name), "unique raw counterpart tree name");
    if (names.length) assert(names.includes(name), "exact selected counterpart tree name");
    tree.set(name, { path: name, mode: match[1], type: match[2], blob: match[3] });
  }
  return tree;
}

function regularEntries(tree, names) {
  return names.map((name) => {
    const row = tree.get(name);
    assert(row?.mode === "100644" && row.type === "blob", "regular counterpart source: " + name);
    return row;
  });
}

function bodies(owned, rows) {
  if (!rows.length) return new Map();
  const packet = git(
    owned.root,
    ["cat-file", "--batch"],
    rows.map((row) => row.blob).join("\n") + "\n",
  );
  const result = new Map();
  let offset = 0;
  for (const row of rows) {
    const stop = packet.indexOf(10, offset);
    assert(stop >= offset, "whole counterpart blob header");
    const header = packet.subarray(offset, stop);
    assert.deepEqual(Buffer.from(header.toString()), header);
    const match = /^([a-f0-9]{40}) blob (0|[1-9][0-9]*)$/u.exec(header.toString());
    assert(match && match[1] === row.blob, "selected counterpart blob identity: " + row.path);
    const size = Number(match[2]);
    assert(Number.isSafeInteger(size));
    const end = stop + 1 + size;
    assert(Number.isSafeInteger(end) && end < packet.length && packet[end] === 10);
    const body = Buffer.from(packet.subarray(stop + 1, end));
    assert.equal(body.length, size);
    assert.equal(object("blob", body), row.blob, "whole counterpart blob bytes: " + row.path);
    result.set(row.path, body);
    offset = end + 1;
  }
  assert.equal(offset, packet.length, "whole counterpart batch response");
  return result;
}

/** Finite Star fixture closure only; this selects files and grants no admission. */
export function listStarCounterpartFiles(owner, source, initialNames) {
  const names = selectedNames(initialNames);
  const owned = context(owner, source);
  const tree = entries(owned);
  regularEntries(tree, names);
  const chosen = new Set(names);
  const prefix = "tests/performance/support/";
  // The original custody runner's computed metadata and directory selections.
  for (const name of tree.keys()) {
    if (
      /^tests\/performance\/support\/warm-type-backed-empty-vue-star-(?:manifest|corrective|part-[0-6])\.json$/u.test(
        name,
      ) ||
      name === prefix + "warm-type-backed-component-value-manifest.json" ||
      name === prefix + "warm-type-backed-component-value-originals.json" ||
      name.startsWith("tests/_fixtures/differential/linter/component-import-value-host-before/") ||
      name.startsWith("tests/_fixtures/differential/lsp/lexical-dollar-host-before/") ||
      (name.startsWith(prefix + "warm-type-backed-git-reader-") && name.endsWith(".json")) ||
      name ===
        "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/component-main-join/successor.json"
    ) {
      assert(safeName(name));
      regularEntries(tree, [name]);
      chosen.add(name);
    }
  }
  let pending = [...chosen];
  while (pending.length) {
    const batch = pending;
    pending = [];
    const raw = bodies(owned, regularEntries(tree, batch));
    for (const name of batch) {
      if (!/\.(?:ts|mjs|js|json)$/u.test(name)) continue;
      const body = raw.get(name).toString();
      const directory = path.posix.dirname(name);
      const candidates = [
        ...body.matchAll(/(?:from\s*|import\s*(?:\(\s*)?)["'](\.[^"']+)["']/gu),
        ...body.matchAll(/new URL\(\s*["'](\.[^"']+)["']\s*,\s*import.meta.url/gu),
      ].map((match) => path.posix.normalize(path.posix.join(directory, match[1])));
      candidates.push(
        ...[
          ...body.matchAll(/["']((?:tests|tools|crates|docs|davinci|\.github)\/[^"'\n]+)["']/gu),
        ].map((match) => match[1]),
        ...[...body.matchAll(/["']([A-Za-z0-9_.-]+\.(?:json|gz|txt))["']/gu)].map((match) =>
          path.posix.join(directory, match[1]),
        ),
      );
      for (const candidate of candidates) {
        if (!safeName(candidate) || chosen.has(candidate)) continue;
        const row = tree.get(candidate);
        if (row?.mode !== "100644" || row.type !== "blob") continue;
        chosen.add(candidate);
        pending.push(candidate);
      }
    }
  }
  return Object.freeze([...chosen].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)));
}

/** Raw literal files in a disposable private Git repository; no authentication credit. */
export function provisionStarSourceCounterpart(owner, source, names) {
  names = selectedNames(names);
  const owned = context(owner, source);
  const tree = entries(owned, names);
  assert.equal(tree.size, names.length, "complete selected counterpart tree");
  const rows = regularEntries(tree, names);
  const raw = bodies(owned, rows);
  const common = git(owned.root, ["rev-parse", "--git-common-dir"]).toString().trim();
  const pool = fs.realpathSync(path.resolve(owned.root, common, "objects"));
  assert(fs.lstatSync(pool).isDirectory() && !/[\r\n]/u.test(pool), "physical owner object pool");
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-counterpart-")));
  const cleanup = () => fs.rmSync(root, { recursive: true, force: true });
  try {
    fs.mkdirSync(path.join(root, ".git/objects/info"), { recursive: true, mode: 0o700 });
    fs.mkdirSync(path.join(root, ".git/refs"), { mode: 0o700 });
    fs.writeFileSync(
      path.join(root, ".git/config"),
      '[core]\nrepositoryformatversion = 0\nbare = false\n[gc]\nauto = 0\n[maintenance]\nauto = false\n[remote "origin"]\nurl = https://github.com/ubugeeei-prod/vize\n',
      { flag: "wx", mode: 0o600 },
    );
    fs.writeFileSync(path.join(root, ".git/HEAD"), source + "\n", { flag: "wx", mode: 0o600 });
    fs.writeFileSync(path.join(root, ".git/objects/info/alternates"), pool + "\n", {
      flag: "wx",
      mode: 0o600,
    });
    for (const row of rows) {
      const target = path.join(root, row.path);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, raw.get(row.path), { flag: "wx", mode: 0o644 });
      fs.chmodSync(target, 0o644);
      const stat = fs.lstatSync(target);
      assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644);
      assert.deepEqual(fs.readFileSync(target), raw.get(row.path));
    }
    assert.equal(git(root, ["rev-parse", "HEAD"]).toString().trim(), source);
    assert.equal(git(root, ["rev-parse", "HEAD^{tree}"]).toString().trim(), owned.tree);
    assert.equal(object("commit", git(root, ["cat-file", "commit", source])), source);
    return Object.freeze({ root, cleanup });
  } catch (error) {
    cleanup();
    throw error;
  }
}
