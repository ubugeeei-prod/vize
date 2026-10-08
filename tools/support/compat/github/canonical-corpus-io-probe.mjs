import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readlinkSync,
  readSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { checkoutIdentity, parseGitlinks } from "./canonical-corpus-identity.mjs";
import { gitObjectId } from "./canonical-corpus-inventory.mjs";
import { cycleOwner } from "./canonical-corpus-logical-walk.mjs";
import { collectorHarness, parseNativeVector } from "./canonical-corpus-native-walk.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const json = (path) => JSON.parse(readFileSync(path, "utf8"));
const save = (path, value) => writeFileSync(path, `${JSON.stringify(value)}\n`);
const rawError = (error) => ({
  code: error.code,
  errno: error.errno,
  syscall: error.syscall,
  message: error.message,
  path: error.path,
});

// Do not use path.join/resolve here: the manifest/../../ spelling is the subject of the probe.
export const literalPath = (root, logical) => `${root}/${logical}`;
export function literalPrefixes(path) {
  assert(path.startsWith("/"), "Diagnostic paths must be absolute");
  return [
    "/",
    ...[...path.matchAll(/\//g)].slice(1).map((match) => path.slice(0, match.index)),
    path,
  ];
}

export function compileProbe(source, directory, rustc = process.env.RUSTC || "rustc") {
  mkdirSync(directory, { recursive: true });
  const { harness } = collectorHarness(source);
  const suffix = readFileSync(new URL("./canonical-corpus-io-probe.rs", import.meta.url));
  const prefix = harness.toString("utf8").replace(/\nfn main\(\) \{[\s\S]*$/, "\n");
  const probe = Buffer.concat([Buffer.from(prefix), suffix]);
  writeFileSync(join(directory, "probe.rs"), probe);
  writeFileSync(join(directory, "collector.rs"), harness);
  const binary = join(directory, "probe");
  const collector = join(directory, "collector");
  for (const [input, output] of [
    ["probe.rs", binary],
    ["collector.rs", collector],
  ])
    execFileSync(rustc, [
      "--edition=2024",
      "-C",
      "debuginfo=0",
      join(directory, input),
      "-o",
      output,
    ]);
  return {
    binary,
    collector,
    sourceSha256: sha256(source),
    probeSha256: sha256(probe),
    binarySha256: sha256(readFileSync(binary)),
    rustc: execFileSync(rustc, ["-vV"]).toString("utf8"),
  };
}

function readExact(fd, length) {
  const bytes = Buffer.alloc(length);
  let offset = 0;
  while (offset < length) {
    const count = readSync(fd, bytes, offset, length - offset, null);
    assert(count > 0, "Truncated Rust diagnostic frame");
    offset += count;
  }
  return bytes;
}

export function probeVariant({ binary, root, vector, expected, directory }) {
  mkdirSync(directory, { recursive: true });
  const input = join(directory, "fixed-vector.nul");
  const collected = join(directory, "collected-vector.nul");
  writeFileSync(input, vector);
  const frames = join(directory, "rust-frames.bin");
  const stdout = openSync(join(directory, "rust.stdout.log"), "w");
  const stderr = openSync(join(directory, "rust.stderr.log"), "w");
  let result;
  try {
    result = spawnSync(binary, [root, input, collected, frames], {
      stdio: ["ignore", stdout, stderr],
    });
  } finally {
    closeSync(stdout);
    closeSync(stderr);
  }
  const paths = parseNativeVector(vector);
  const rows = [];
  const nodeSequence = createHash("sha256");
  const rustSequence = { file_open: createHash("sha256"), read_to_string: createHash("sha256") };
  const wire = createHash("sha256");
  const fd = openSync(frames, "r");
  let frameCount = 0;
  try {
    for (const [index, logical] of paths.entries()) {
      const path = literalPath(root, logical);
      const row = { index, logical, path, node: {}, rust: {}, matches: {} };
      let nodeBytes;
      try {
        nodeBytes = readFileSync(path);
        row.node = {
          status: "ok",
          bytes: nodeBytes.length,
          sha256: sha256(nodeBytes),
          blob: gitObjectId("blob", nodeBytes),
        };
        nodeSequence.update(logical).update("\0").update(nodeBytes);
        if (expected)
          row.node.matchesSnapshot =
            row.node.sha256 === expected[index][1] &&
            row.node.bytes === expected[index][2] &&
            row.node.blob === expected[index][3];
      } catch (error) {
        row.node = { status: "error", ...rawError(error) };
      }
      for (const operation of ["stat", "open64", "file_open", "read_to_string"]) {
        const headerLengthBytes = readExact(fd, 8);
        const headerBytes = readExact(fd, Number(headerLengthBytes.readBigUInt64LE()));
        const dataLengthBytes = readExact(fd, 8);
        const bytes = readExact(fd, Number(dataLengthBytes.readBigUInt64LE()));
        wire.update(headerLengthBytes).update(headerBytes).update(dataLengthBytes).update(bytes);
        const [actualIndex, actualOperation, status, ...detail] = headerBytes
          .toString("utf8")
          .split("\t");
        assert.equal(
          Number(actualIndex),
          index,
          "Rust diagnostic omitted or reordered an original input",
        );
        assert.equal(
          actualOperation,
          operation,
          "Rust diagnostic omitted or reordered an operation",
        );
        assert(["ok", "error", "unsupported"].includes(status), "Foreign diagnostic status");
        assert(
          status !== "unsupported" || (operation === "open64" && process.platform !== "linux"),
          "Required Linux operation was not executed",
        );
        row.rust[operation] = { status, detail: detail.join("\t"), bytes: bytes.length };
        if (["file_open", "read_to_string"].includes(operation) && status === "ok") {
          row.rust[operation].sha256 = sha256(bytes);
          rustSequence[operation].update(logical).update("\0").update(bytes);
          row.matches[operation] = nodeBytes !== undefined && bytes.equals(nodeBytes);
        }
        frameCount += 1;
      }
      rows.push(row);
    }
    const extra = Buffer.alloc(1);
    assert.equal(readSync(fd, extra, 0, 1, null), 0, "Extra Rust diagnostic input");
  } finally {
    closeSync(fd);
  }
  const prefixes = new Set(paths.flatMap((path) => literalPrefixes(literalPath(root, path))));
  const prefixRows = [...prefixes].map((path) => {
    try {
      const stat = lstatSync(path, { bigint: true });
      return {
        path,
        status: "ok",
        type: stat.isSymbolicLink()
          ? "symlink"
          : stat.isDirectory()
            ? "directory"
            : stat.isFile()
              ? "file"
              : "other",
        dev: String(stat.dev),
        ino: String(stat.ino),
        bytes: String(stat.size),
        ...(stat.isSymbolicLink() ? { target: readlinkSync(path) } : {}),
      };
    } catch (error) {
      return { path, status: "error", ...rawError(error) };
    }
  });
  writeFileSync(
    join(directory, "operations.jsonl"),
    rows.map((row) => JSON.stringify(row)).join("\n") + "\n",
  );
  writeFileSync(
    join(directory, "prefix-lstat.jsonl"),
    prefixRows.map((row) => JSON.stringify(row)).join("\n") + "\n",
  );
  const summary = {
    root,
    selected: paths.length,
    frameCount,
    status: result.status,
    signal: result.signal,
    spawnError: result.error ? rawError(result.error) : null,
    vectorSha256: sha256(vector),
    collectedVectorMatches: readFileSync(collected).equals(vector),
    nodeSequenceSha256: nodeSequence.digest("hex"),
    rustSequenceSha256: Object.fromEntries(
      Object.entries(rustSequence).map(([name, hash]) => [name, hash.digest("hex")]),
    ),
    framesSha256: wire.digest("hex"),
    failures: rows.filter(
      (row) =>
        row.node.status !== "ok" ||
        row.node.matchesSnapshot === false ||
        Object.values(row.rust).some((operation) => operation.status === "error") ||
        Object.values(row.matches).some((matches) => !matches),
    ).length,
    prefixFailures: prefixRows.filter((row) => row.status === "error").length,
  };
  summary.success =
    summary.status === 0 &&
    summary.collectedVectorMatches &&
    summary.failures === 0 &&
    summary.prefixFailures === 0;
  save(join(directory, "summary.json"), summary);
  // Successful complete bytes were compared directly. Keep the original frames on failure,
  // including any partial read payload, alongside every operation and the wire digest.
  if (summary.success) unlinkSync(frames);
  return summary;
}

function pinnedMinimal(cwd, directory, collector) {
  const repository = join(cwd, cycleOwner.repository);
  const sourcePath = "packages/frontend/src/components/Playback/UpNext.vue";
  const tree = (path) =>
    execFileSync("git", ["-C", repository, "ls-tree", "-z", cycleOwner.revision, "--", path], {
      encoding: "utf8",
    });
  assert.equal(
    tree(sourcePath),
    `100644 blob f23ff4de9fa28533d163d8123dd78ca54622abef\t${sourcePath}\0`,
  );
  assert.equal(tree("packaging/deb/root"), `120000 blob ${cycleOwner.blob}\tpackaging/deb/root\0`);
  const source = execFileSync("git", [
    "-C",
    repository,
    "show",
    `${cycleOwner.revision}:${sourcePath}`,
  ]);
  const link = execFileSync("git", ["-C", repository, "cat-file", "blob", cycleOwner.blob]);
  assert.equal(gitObjectId("blob", source), "f23ff4de9fa28533d163d8123dd78ca54622abef");
  assert.equal(source.length, 3748);
  assert.equal(link.toString("utf8"), cycleOwner.target);
  const workspace = join(directory, "minimal-workspace");
  const root = join(workspace, "tests/_fixtures/_git");
  mkdirSync(join(workspace, "tests/davinci_test_support"), { recursive: true });
  mkdirSync(dirname(join(root, "jellyfin-vue", sourcePath)), { recursive: true });
  mkdirSync(join(root, "jellyfin-vue/packaging/deb"), { recursive: true });
  writeFileSync(join(root, "jellyfin-vue", sourcePath), source);
  symlinkSync(link.toString("utf8"), join(root, "jellyfin-vue/packaging/deb/root"));
  const vector = execFileSync(collector, [root]);
  const sourceSha256 = sha256(source);
  const sourceBlob = gitObjectId("blob", source);
  return {
    workspace,
    root,
    vector,
    expected: parseNativeVector(vector).map((path) => [
      path,
      sourceSha256,
      source.length,
      sourceBlob,
    ]),
    sourceSha256,
    sourceBlob,
    cycle: cycleOwner,
  };
}

export function runCanonicalProbe(cwd, output, env = process.env) {
  assert.equal(process.platform, "linux", "Complete hosted qualification requires Linux");
  const artifact = join(cwd, "real-project-davinci-dom-corpus");
  const identity = json(join(artifact, "identity.json"));
  const actual = checkoutIdentity(cwd, env);
  for (const key of ["sha", "tree", "runId", "attempt", "repository"])
    assert.equal(identity[key], actual[key], `Foreign diagnostic ${key}`);
  assert.deepEqual(
    identity.gitlinks,
    parseGitlinks(
      execFileSync("git", ["ls-tree", "-rz", "HEAD", "--", "tests/_fixtures/_git"], {
        cwd,
        encoding: "utf8",
      }),
    ),
  );
  assert.equal(identity.gitlinksSha256, sha256(JSON.stringify(identity.gitlinks)));
  assert.equal(identity.gitlinks.length, 148);
  assert(
    identity.gitlinks.some(
      (row) => row.path === cycleOwner.repository && row.sha === cycleOwner.revision,
    ),
    "Foreign Jellyfin pin",
  );
  const vector = readFileSync(join(artifact, "native-vector.nul"));
  const paths = parseNativeVector(vector);
  const expected = json(join(artifact, "files.json"));
  assert.equal(identity.filesSha256, sha256(JSON.stringify(expected)));
  assert.equal(paths.length, 44367);
  assert.equal(paths.filter((path) => path.startsWith("jellyfin-vue/")).length, 5576);
  assert.deepEqual(
    paths,
    expected.map(([path]) => path),
  );
  assert.equal(sha256(vector), identity.nativeWalk.vectorSha256);
  const source = readFileSync(join(artifact, "native-collector-source.rs"));
  assert.equal(sha256(source), identity.nativeWalk.sourceSha256);
  assert.deepEqual(source, readFileSync(join(cwd, "tests/davinci_test_support/src/corpus.rs")));
  const tools = compileProbe(source, join(output, "harness"));
  const minimal = pinnedMinimal(cwd, output, tools.collector);
  const cases = [
    { name: "full-short", root: resolve(cwd, "tests/_fixtures/_git"), vector, expected },
    {
      name: "full-manifest",
      root: `${cwd}/tests/davinci_test_support/../../tests/_fixtures/_git`,
      vector,
      expected,
    },
    {
      name: "minimal-short",
      root: minimal.root,
      vector: minimal.vector,
      expected: minimal.expected,
    },
    {
      name: "minimal-manifest",
      root: `${minimal.workspace}/tests/davinci_test_support/../../tests/_fixtures/_git`,
      vector: minimal.vector,
      expected: minimal.expected,
    },
  ];
  save(join(output, "custody.json"), {
    identity,
    tools,
    minimal: { ...minimal, vector: undefined, expected: undefined },
    node: process.version,
    platform: process.platform,
    architecture: process.arch,
  });
  const summaries = [];
  for (const probe of cases) {
    console.log(
      `Canonical IO diagnostic ${probe.name}: ${parseNativeVector(probe.vector).length} complete original inputs`,
    );
    summaries.push({
      name: probe.name,
      ...probeVariant({ ...probe, binary: tools.binary, directory: join(output, probe.name) }),
    });
  }
  const receipt = {
    schema: "vize.canonical-corpus-io-diagnostic",
    version: 1,
    ...actual,
    files: paths.length,
    jellyfinFiles: 5576,
    gitlinks: 148,
    summaries,
    success: summaries.every((summary) => summary.success),
  };
  save(join(output, "receipt.json"), receipt);
  console.log(JSON.stringify(receipt));
  return receipt;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const cwd = process.cwd();
  const output = join(cwd, "real-project-davinci-dom-corpus/io-diagnostic");
  mkdirSync(output, { recursive: true });
  try {
    if (!runCanonicalProbe(cwd, output).success) process.exitCode = 1;
  } catch (error) {
    writeFileSync(join(output, "fatal.log"), `${error.stack}\n`);
    console.error(error);
    process.exitCode = 1;
  }
}
