import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { publicReader } from "../public_acceptance/http.ts";

const limit = 512 * 1024 * 1024;
const maximumEntries = 100_000;
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const text = (bytes: Uint8Array) => new TextDecoder("utf-8", { fatal: true }).decode(bytes);
export type ArchiveEntry = { path: string; sha256: string; bytes: number };
export type ArchiveManifest = {
  name: string;
  version: string;
  bytes: number;
  sha256: string;
  content: string;
};
export type ArchiveEvidence = {
  archive: { sha256: string; sha512: string; bytes: number };
  archiveEntries: ArchiveEntry[];
  archiveManifest: ArchiveManifest;
};
export type InstalledArchive = ArchiveEvidence & {
  name: string;
  version: string;
  directory: string;
  integrity: string;
  tarball: string;
  manifestSha256: string;
  installedFiles: ArchiveEntry[];
  native?: { path: string; sha256: string };
};

function relativeName(value: string, directory = false) {
  const name = directory && value.endsWith("/") ? value.slice(0, -1) : value;
  assert.ok(name.length > 0 && name.length <= 4096, "bounded nonempty archive path required");
  assert.ok(!/[\\:]/u.test(name), "unsafe archive path");
  for (const character of name)
    assert.ok(
      character.charCodeAt(0) >= 32 && character.charCodeAt(0) !== 127,
      "archive control character",
    );
  for (const component of name.split("/")) {
    assert.ok(component && component !== "." && component !== "..", "archive traversal or alias");
    assert.equal(component.normalize("NFC"), component, "noncanonical archive path");
    assert.ok(!/[. ]$/u.test(component), "Windows archive path alias");
    assert.ok(
      !/^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/iu.test(component),
      "Windows device path",
    );
  }
  return name;
}

function octal(bytes: Uint8Array) {
  const value = text(bytes).replaceAll("\0", " ").trim();
  assert.match(value, /^[0-7]+$/u, "canonical octal TAR field required");
  const result = Number.parseInt(value, 8);
  assert.ok(Number.isSafeInteger(result) && result <= limit, "bounded TAR size required");
  return result;
}

function field(bytes: Uint8Array) {
  const zero = bytes.indexOf(0);
  if (zero < 0) return text(bytes);
  assert.ok(
    bytes.subarray(zero).every((value) => value === 0),
    "nonzero bytes after TAR name terminator",
  );
  return text(bytes.subarray(0, zero));
}

function pax(bytes: Uint8Array) {
  const records = Object.create(null) as Record<string, string>;
  let offset = 0;
  while (offset < bytes.length) {
    const space = bytes.indexOf(32, offset);
    assert.ok(space > offset && space - offset <= 10, "invalid PAX length");
    const lengthText = text(bytes.subarray(offset, space));
    assert.match(lengthText, /^[1-9][0-9]*$/u);
    const length = Number(lengthText);
    assert.ok(
      length > space - offset + 2 && offset + length <= bytes.length,
      "truncated PAX record",
    );
    assert.equal(bytes[offset + length - 1], 10, "PAX newline required");
    const record = text(bytes.subarray(space + 1, offset + length - 1));
    const equals = record.indexOf("=");
    assert.ok(equals > 0, "PAX key required");
    const key = record.slice(0, equals);
    assert.ok(!Object.hasOwn(records, key), "duplicate PAX key");
    assert.ok(
      !["linkpath", "size", "GNU.sparse.map", "GNU.sparse.name"].includes(key),
      "unsupported PAX storage override",
    );
    assert.ok(!key.startsWith("GNU.sparse."), "sparse TAR entries are unsupported");
    records[key] = record.slice(equals + 1);
    offset += length;
  }
  return records;
}

/** The network reader and offline fixtures use the same strict TAR implementation. */
export function inspectArchive(bytes: Uint8Array, integrity: string): ArchiveEvidence {
  assert.ok(
    bytes.byteLength > 0 && bytes.byteLength <= limit,
    "bounded nonempty compressed archive required",
  );
  assert.match(integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u, "exact SHA512 SRI required");
  const sha512 = createHash("sha512").update(bytes).digest("base64");
  assert.equal(`sha512-${sha512}`, integrity, "public npm archive SRI mismatch");
  const tar = gunzipSync(bytes, { maxOutputLength: limit });
  assert.ok(tar.length >= 1024 && tar.length % 512 === 0, "whole TAR blocks required");
  const entries: ArchiveEntry[] = [];
  const names = new Map<string, { name: string; file: boolean; explicit: boolean }>();
  let manifest: ArchiveManifest | undefined;
  let offset = 0;
  let local: Record<string, string> | undefined;
  let longName: string | undefined;
  let terminated = false;
  let headers = 0;
  const register = (name: string, file: boolean) => {
    const components = name.split("/");
    for (let index = 0; index < components.length; index++) {
      const prefix = components.slice(0, index + 1).join("/");
      const key = prefix.toLowerCase();
      const final = index === components.length - 1;
      const prior = names.get(key);
      assert.ok(!prior || prior.name === prefix, "case-insensitive archive path alias");
      assert.ok(!prior?.file, "duplicate file or file used as directory");
      if (final)
        assert.ok(file ? !prior : !prior?.explicit, "duplicate or conflicting archive entry");
      names.set(key, {
        name: prefix,
        file: final && file,
        explicit: final || prior?.explicit === true,
      });
    }
  };
  while (offset < tar.length) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every((value) => value === 0)) {
      assert.ok(
        tar.length - offset >= 1024 && tar.subarray(offset).every((value) => value === 0),
        "two zero TAR end blocks required",
      );
      terminated = true;
      break;
    }
    assert.ok(++headers <= maximumEntries, "too many TAR entries");
    const stored = octal(header.subarray(148, 156));
    const sum = header.reduce(
      (total, value, index) => total + (index >= 148 && index < 156 ? 32 : value),
      0,
    );
    assert.equal(sum, stored, "TAR header checksum mismatch");
    const size = octal(header.subarray(124, 136));
    const end = offset + 512 + size;
    assert.ok(end <= tar.length, "truncated TAR entry");
    const body = tar.subarray(offset + 512, end);
    const next = offset + 512 + Math.ceil(size / 512) * 512;
    assert.ok(next <= tar.length, "truncated TAR padding");
    assert.ok(
      tar.subarray(end, next).every((value) => value === 0),
      "nonzero TAR entry padding",
    );
    offset = next;
    const type = String.fromCharCode(header[156]);
    if (type === "x") {
      assert.ok(!local && !longName, "overlapping TAR name overrides");
      local = pax(body);
      continue;
    }
    if (type === "L") {
      assert.ok(!local && !longName, "overlapping TAR name overrides");
      assert.ok(body.length > 0 && body.at(-1) === 0, "terminated GNU long name required");
      longName = field(body);
      continue;
    }
    assert.ok(
      type === "0" || type === "\0" || type === "5",
      "only normal TAR files and directories are accepted",
    );
    assert.equal(field(header.subarray(157, 257)), "", "TAR link target is forbidden");
    const magic = field(header.subarray(257, 263));
    assert.ok(["", "ustar", "ustar "].includes(magic), "unsupported TAR format");
    const prefix = magic === "ustar" ? field(header.subarray(345, 500)) : "";
    const name =
      local?.path ?? longName ?? [prefix, field(header.subarray(0, 100))].filter(Boolean).join("/");
    local = undefined;
    longName = undefined;
    const normalized = relativeName(name, type === "5");
    assert.ok(
      normalized === "package" || normalized.startsWith("package/"),
      "npm package/ root required",
    );
    register(normalized, type !== "5");
    if (type === "5") {
      assert.equal(size, 0, "TAR directories must be empty");
      continue;
    }
    assert.ok(normalized !== "package", "npm package root must be a directory");
    const relative = normalized.slice(8);
    entries.push({ path: relative, sha256: hash(body), bytes: size });
    if (relative === "package.json") {
      const content = text(body);
      const value: unknown = JSON.parse(content);
      assert.ok(value && typeof value === "object" && !Array.isArray(value));
      const identity = value as Record<string, unknown>;
      assert.ok(typeof identity.name === "string" && typeof identity.version === "string");
      manifest = {
        name: identity.name,
        version: identity.version,
        content,
        bytes: size,
        sha256: hash(body),
      };
    }
  }
  assert.ok(terminated && !local && !longName, "complete TAR with no pending override required");
  assert.ok(manifest && entries.length > 0, "npm package manifest required");
  entries.sort((left, right) => left.path.localeCompare(right.path));
  return {
    archive: { sha256: hash(bytes), sha512, bytes: bytes.byteLength },
    archiveEntries: entries,
    archiveManifest: manifest,
  };
}

/** Public npm GET only, with one deadline covering headers and streamed bytes. */
export async function readArchive(url: string, integrity: string): Promise<ArchiveEvidence> {
  const parsed = new URL(url);
  assert.equal(parsed.origin, "https://registry.npmjs.org");
  assert.ok(!parsed.username && !parsed.password && !parsed.hash && !parsed.search);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 30_000);
  try {
    const response = await fetch(parsed, {
      signal: controller.signal,
      redirect: "error",
      credentials: "omit",
    });
    assert.ok(response.ok && response.body, `public npm archive HTTP ${response.status}`);
    const announced = response.headers.get("content-length");
    if (announced !== null) assert.ok(Number(announced) > 0 && Number(announced) <= limit);
    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let length = 0;
    try {
      for (;;) {
        const part = await reader.read();
        if (part.done) break;
        length += part.value.byteLength;
        assert.ok(length <= limit, "compressed archive exceeds 512MiB");
        chunks.push(part.value);
      }
    } finally {
      await reader.cancel();
      reader.releaseLock();
    }
    return inspectArchive(Buffer.concat(chunks, length), integrity);
  } finally {
    controller.abort();
    clearTimeout(timer);
  }
}

/** Every shipped regular file is read from its actual installed package directory. */
export function observeInstalledArchive(directory: string, entries: ArchiveEntry[]) {
  assert.ok(entries.length > 0 && entries.length <= maximumEntries);
  assert.ok(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
  const root = fs.realpathSync(directory);
  const installedFiles = entries.map((entry) => {
    const relative = relativeName(entry.path);
    let cursor = root;
    for (const component of relative.split("/")) {
      cursor = path.join(cursor, component);
      const stat = fs.lstatSync(cursor);
      assert.ok(!stat.isSymbolicLink(), "installed archive path cannot contain symlinks");
      if (cursor !== path.join(root, relative)) assert.ok(stat.isDirectory());
    }
    assert.ok(
      fs.realpathSync(cursor).startsWith(root + path.sep),
      "installed file escaped package root",
    );
    const descriptor = fs.openSync(cursor, fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0));
    try {
      const before = fs.fstatSync(descriptor);
      assert.ok(
        before.isFile() && before.size === entry.bytes && before.size <= limit,
        "installed regular file size mismatch",
      );
      const bytes = fs.readFileSync(descriptor);
      const after = fs.fstatSync(descriptor);
      assert.equal(after.ino, before.ino);
      assert.equal(after.size, before.size);
      assert.equal(after.mtimeMs, before.mtimeMs);
      return { path: relative, sha256: hash(bytes), bytes: bytes.length };
    } finally {
      fs.closeSync(descriptor);
    }
  });
  assert.deepEqual(
    installedFiles,
    entries,
    "installed shipped files differ from the verified public archive",
  );
  return { installedFiles };
}

/** Offline comparison of signed inspector output against actual runtime observations. */
export function verifyInstalledArchive(record: InstalledArchive) {
  assert.ok(record && typeof record === "object");
  const paths =
    /^[a-z]:[\\/]/iu.test(record.directory) || record.directory.startsWith("\\\\")
      ? path.win32
      : path.posix;
  assert.ok(paths.isAbsolute(record.directory));
  assert.equal(
    paths.normalize(record.directory),
    record.directory,
    "canonical installed directory required",
  );
  assert.match(record.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u);
  assert.equal(record.integrity, `sha512-${record.archive.sha512}`);
  assert.match(record.archive.sha256, /^[a-f0-9]{64}$/u);
  assert.ok(
    Number.isSafeInteger(record.archive.bytes) &&
      record.archive.bytes > 0 &&
      record.archive.bytes <= limit,
  );
  assert.ok(
    Array.isArray(record.archiveEntries) &&
      record.archiveEntries.length > 0 &&
      record.archiveEntries.length <= maximumEntries,
  );
  const seen = new Set<string>();
  let total = 0;
  for (const entry of record.archiveEntries) {
    relativeName(entry.path);
    assert.ok(!seen.has(entry.path.toLowerCase()), "duplicate installed archive alias");
    seen.add(entry.path.toLowerCase());
    assert.match(entry.sha256, /^[a-f0-9]{64}$/u);
    assert.ok(Number.isSafeInteger(entry.bytes) && entry.bytes >= 0);
    total += entry.bytes;
    assert.ok(total <= limit);
  }
  assert.deepEqual(
    record.installedFiles,
    record.archiveEntries,
    "all shipped files must match public archive bytes",
  );
  const manifest = record.archiveManifest;
  assert.equal(manifest.sha256, hash(Buffer.from(manifest.content)));
  assert.equal(manifest.bytes, Buffer.byteLength(manifest.content));
  assert.deepEqual(
    record.archiveEntries.find((entry) => entry.path === "package.json"),
    { path: "package.json", bytes: manifest.bytes, sha256: manifest.sha256 },
  );
  const identity: unknown = JSON.parse(manifest.content);
  assert.ok(identity && typeof identity === "object" && !Array.isArray(identity));
  assert.equal((identity as Record<string, unknown>).name, record.name);
  assert.equal((identity as Record<string, unknown>).version, record.version);
  assert.equal(manifest.name, record.name);
  assert.equal(manifest.version, record.version);
  assert.equal(
    record.manifestSha256,
    manifest.sha256,
    "runtime manifest must match public archive",
  );
  if (record.native) {
    assert.ok(paths.isAbsolute(record.directory) && paths.isAbsolute(record.native.path));
    assert.equal(
      paths.normalize(record.native.path),
      record.native.path,
      "canonical native provider path required",
    );
    const relative = paths
      .relative(record.directory, record.native.path)
      .split(paths.sep)
      .join("/");
    relativeName(relative);
    assert.ok(relative.endsWith(".node"), "native provider must be a shipped .node file");
    const entry = record.archiveEntries.find((value) => value.path === relative);
    assert.ok(entry, "loaded native provider must belong to its own public package archive");
    assert.equal(
      record.native.sha256,
      entry.sha256,
      "loaded native provider bytes differ from public archive",
    );
  }
}

type RevalidationDependencies = {
  metadata?: (name: string, version: string) => Promise<unknown>;
  inspect?: (url: string, integrity: string) => Promise<ArchiveEvidence>;
};
const publicArchives = new Map<string, Promise<ArchiveEvidence>>();
function publicInspection(url: string, integrity: string) {
  const key = `${url}\n${integrity}`;
  let pending = publicArchives.get(key);
  if (!pending) {
    pending = readArchive(url, integrity).catch((error) => {
      publicArchives.delete(key);
      throw error;
    });
    publicArchives.set(key, pending);
  }
  return pending;
}

/** Each finalizer/verifier independently recomputes the actual public archive index. */
export async function revalidatePublicArchives(
  records: InstalledArchive[],
  native: { path: string; sha256: string },
  dependencies: RevalidationDependencies = {},
) {
  assert.ok(
    native && typeof native.path === "string" && typeof native.sha256 === "string",
    "actual loaded native observation required",
  );
  assert.ok(
    records.length > 0 && records.length <= 32,
    "bounded public package observations required",
  );
  const names = new Set<string>();
  const providers = records.filter((record) => record.name.startsWith("@vizejs/native-"));
  assert.equal(providers.length, 1, "exactly one actual platform native provider required");
  assert.equal(
    new Set(records.map((record) => record.version)).size,
    1,
    "one public release version required",
  );
  const reader = publicReader();
  const metadata =
    dependencies.metadata ??
    ((name: string, version: string) =>
      reader.json(`https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`));
  const inspect = dependencies.inspect ?? publicInspection;
  const cache = new Map<string, ArchiveEvidence>();
  const observations = [];
  for (const record of records) {
    assert.match(record.name, /^(?:vize|@vizejs\/[a-z0-9-]+)$/u);
    assert.match(record.version, /^0\.[1-9][0-9]*\.0$/u);
    assert.ok(!names.has(record.name), "duplicate public package observation");
    names.add(record.name);
    const value = await metadata(record.name, record.version);
    assert.ok(value && typeof value === "object" && !Array.isArray(value));
    const published = value as Record<string, unknown>;
    assert.equal(published.name, record.name);
    assert.equal(published.version, record.version);
    assert.ok(published.dist && typeof published.dist === "object");
    const dist = published.dist as Record<string, unknown>;
    assert.equal(
      record.tarball,
      dist.tarball,
      "claimed archive URL differs from the public registry",
    );
    assert.equal(record.integrity, dist.integrity, "claimed SRI differs from the public registry");
    const url = new URL(record.tarball);
    assert.equal(url.origin, "https://registry.npmjs.org");
    assert.ok(!url.username && !url.password && !url.hash && !url.search);
    const key = `${record.tarball}\n${record.integrity}`;
    let actual = cache.get(key);
    if (!actual) {
      actual = await inspect(record.tarball, record.integrity);
      cache.set(key, actual);
    }
    assert.deepEqual(record.archive, actual.archive, "actual public compressed archive differs");
    assert.deepEqual(
      record.archiveEntries,
      actual.archiveEntries,
      "actual public archive entries differ",
    );
    assert.deepEqual(
      record.archiveManifest,
      actual.archiveManifest,
      "actual public archive manifest differs",
    );
    verifyInstalledArchive({ ...record, native: record === providers[0] ? native : undefined });
    observations.push({
      name: record.name,
      version: record.version,
      metadata: published,
      ...actual,
    });
  }
  return observations;
}
