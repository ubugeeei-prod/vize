/** Untimed custody of observed native members; this does not certify graph closure. */
import assert from "node:assert/strict";
import { createHash, randomUUID } from "node:crypto";
import {
  closeSync,
  constants,
  fstatSync,
  linkSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join, parse, relative, resolve, sep } from "node:path";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const append = (base, tail) => (tail ? `${base}${base.endsWith(sep) ? "" : sep}${tail}` : base);
const revision = (stat) =>
  Object.fromEntries(
    ["dev", "ino", "mode", "size", "mtimeNs", "ctimeNs"].map((key) => [key, String(stat[key])]),
  );

/** Retain every traversed symlink, including links inside another link's target. */
function linksFor(path) {
  const links = [];
  let pending = path;
  for (let depth = 0; depth < 128; depth++) {
    const root = parse(pending).root;
    const parts = pending
      .slice(root.length)
      .split(sep === "\\" ? /[\\/]/u : sep)
      .filter(Boolean);
    let prefix = root;
    let redirected = false;
    for (const [index, part] of parts.entries()) {
      prefix = append(prefix, part);
      const stat = lstatSync(prefix, { bigint: true });
      if (!stat.isSymbolicLink()) continue;
      const target = readlinkSync(prefix);
      links.push({ path: prefix, target, revision: revision(stat) });
      const destination = isAbsolute(target) ? target : append(dirname(prefix), target);
      pending = append(destination, parts.slice(index + 1).join(sep));
      redirected = true;
      break;
    }
    if (!redirected) return links;
  }
  throw new Error(`unsupported symlink chain: ${path}`);
}

/** Two identity reads fence the byte read; atime changes caused by reading are excluded. */
function readStable(path) {
  const identity = () => ({
    realPath: realpathSync.native(path),
    links: linksFor(path),
    revision: revision(lstatSync(realpathSync.native(path), { bigint: true })),
  });
  const before = identity();
  const bytes = readFileSync(path);
  assert.deepEqual(identity(), before, `file identity changed while reading: ${path}`);
  assert.equal(String(bytes.length), before.revision.size, `file size changed: ${path}`);
  return { bytes, record: { path, ...before, bytes: bytes.length, sha256: sha(bytes) } };
}

function ensureDirectory(path) {
  mkdirSync(path, { recursive: true });
  const stat = lstatSync(path);
  assert(stat.isDirectory() && !stat.isSymbolicLink(), `unsafe archive directory: ${path}`);
}

/** A completed temporary inode is published atomically, allowing concurrent shard dedup. */
function storeObject(directory, bytes) {
  const hash = sha(bytes);
  const object = join(hash.slice(0, 2), hash);
  ensureDirectory(directory);
  ensureDirectory(join(directory, hash.slice(0, 2)));
  const file = join(directory, object);
  const temporary = `${file}.${process.pid}.${randomUUID()}.tmp`;
  writeFileSync(temporary, bytes, { flag: "wx", mode: 0o400 });
  try {
    try {
      linkSync(temporary, file);
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
    }
    verifyObject(file, bytes);
  } finally {
    unlinkSync(temporary);
  }
  return object;
}

function verifyObject(file, bytes) {
  const stat = lstatSync(file, { bigint: true });
  assert(stat.isFile() && !stat.isSymbolicLink(), `unsafe archive object: ${file}`);
  const fd = openSync(file, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
  try {
    const opened = fstatSync(fd, { bigint: true });
    assert.equal(opened.dev, stat.dev, `archive object replaced: ${file}`);
    assert.equal(opened.ino, stat.ino, `archive object replaced: ${file}`);
    assert(readFileSync(fd).equals(bytes), `archive object bytes disagree: ${file}`);
  } finally {
    closeSync(fd);
  }
}

function listing(stem, phase) {
  const streams = {};
  let stdout = Buffer.alloc(0);
  for (const stream of ["stdout", "stderr"]) {
    const file = `${stem}.membership-${phase}.${stream}.txt`;
    const bytes = readFileSync(file);
    streams[stream] = { file: basename(file), bytes: bytes.length, sha256: sha(bytes) };
    if (stream === "stdout") stdout = bytes;
    else assert.equal(bytes.length, 0, "unexpected native membership stderr");
  }
  const text = stdout.toString("utf8");
  assert(Buffer.from(text).equals(stdout), "unsupported non-UTF-8 native listing");
  const rows = text.split("\n");
  if (rows.at(-1) === "") rows.pop();
  const paths = rows.map((row) => (row.endsWith("\r") ? row.slice(0, -1) : row));
  assert(
    paths.every((path) => path.length > 0),
    "blank native member path",
  );
  return { streams, paths };
}

/**
 * @param {{directory: string, stem: string, cwd: string, config: string,
 * configBytes: Buffer, runtime: string, args: string[], objectsDirectory?: string}} options
 */
export function createNativeGraphArchive(options) {
  const { directory, stem, cwd, config, configBytes, runtime, args } = options;
  const nativeArgs = [...args];
  assert([directory, stem, cwd, config, runtime].every(isAbsolute), "absolute context required");
  assert.equal(dirname(stem), directory, "archive stem must belong to its directory");
  ensureDirectory(directory);
  const storage = realpathSync.native(directory);
  const requestedObjects = options.objectsDirectory ?? join(storage, "graph-objects");
  assert(isAbsolute(requestedObjects), "absolute object store required");
  ensureDirectory(requestedObjects);
  const objectStorage = realpathSync.native(requestedObjects);
  const initialConfig = readStable(config);
  const initialRuntime = readStable(runtime).record;
  assert(initialConfig.bytes.equals(configBytes), "config changed before archive started");
  const rawConfig = `${basename(stem)}.graph.config.json`;
  writeFileSync(join(storage, rawConfig), configBytes, { flag: "wx" });
  const parsedConfig = JSON.parse(configBytes.toString("utf8"));
  const declaredRootSelection = {};
  for (const key of ["files", "include", "exclude", "references"]) {
    if (Object.hasOwn(parsedConfig, key)) declaredRootSelection[key] = parsedConfig[key];
  }
  let before = null;
  let beforeListing = null;
  let finished = false;

  /** @param {{path: string, bytes: number, sha256: string}[]} members */
  function capture(members, phase) {
    const listed = listing(stem, phase);
    assert.equal(listed.paths.length, members.length, "native listing/member count changed");
    const rows = members.map((member, index) => {
      const reportedPath = listed.paths[index];
      assert.equal(member.path, resolve(cwd, reportedPath), "native member path/order mismatch");
      // Preserve the raw spelling for filesystem access: normalizing a symlink/.. is unsafe.
      const accessPath = isAbsolute(reportedPath) ? reportedPath : append(cwd, reportedPath);
      const captured = readStable(accessPath);
      assert.equal(captured.record.bytes, member.bytes, `member length changed: ${member.path}`);
      assert.equal(captured.record.sha256, member.sha256, `member bytes changed: ${member.path}`);
      const object = relative(
        storage,
        join(objectStorage, storeObject(objectStorage, captured.bytes)),
      );
      return { index, reportedPath, ...captured.record, object };
    });
    return { rows, listed };
  }

  return {
    /** @param {{path: string, bytes: number, sha256: string}[]} members */
    captureMembers(members) {
      assert(!before && !finished, "before membership already captured");
      const captured = capture(members, "before");
      before = captured.rows;
      beforeListing = captured.listed;
      return before;
    },
    /** @param {{path: string, bytes: number, sha256: string}[]} members */
    finish(members) {
      assert(before && beforeListing && !finished, "before membership required; finish once");
      const after = capture(members, "after");
      assert.deepEqual(after.rows, before, "native member order/bytes/identity changed");
      for (const stream of ["stdout", "stderr"]) {
        assert.equal(
          after.listed.streams[stream].sha256,
          beforeListing.streams[stream].sha256,
          `native membership ${stream} changed`,
        );
      }
      assert.deepEqual(readStable(config).record, initialConfig.record, "config identity changed");
      assert.deepEqual(
        readStable(runtime).record,
        initialRuntime,
        "native runtime identity changed",
      );
      assert.deepEqual(args, nativeArgs, "native arguments changed during capture");
      assert(
        readFileSync(join(storage, rawConfig)).equals(initialConfig.bytes),
        "archived config changed",
      );
      for (const member of before) {
        assert.deepEqual(
          readStable(member.path).record,
          (({ index: _index, reportedPath: _reportedPath, object: _object, ...record }) => record)(
            member,
          ),
          "native member changed after listing",
        );
        verifyObject(join(storage, member.object), readFileSync(member.path));
      }
      const record = {
        schemaVersion: 1,
        authority: "observed native listFilesOnly byte custody; graph closure unclaimed",
        cwd,
        args: nativeArgs,
        config: { ...initialConfig.record, rawFile: rawConfig, declaredRootSelection },
        runtime: initialRuntime,
        objectStore: { path: objectStorage, relativeDirectory: relative(storage, objectStorage) },
        listings: { before: beforeListing.streams, after: after.listed.streams },
        membersBefore: before,
        membersAfter: after.rows,
        nativeGraphBytesUnchanged: true,
        graphClosureClaimed: false,
      };
      writeFileSync(`${stem}.graph.json`, `${JSON.stringify(record, null, 2)}\n`, { flag: "wx" });
      finished = true;
      return record;
    },
  };
}
