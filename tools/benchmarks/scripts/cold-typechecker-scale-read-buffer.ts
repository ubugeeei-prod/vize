/** Separate instrumented source overlay; never a primary timing image. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { buildColdScaleBinary } from "./cold-typechecker-scale-build.ts";

const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const projectFile = "crates/vize_canon/src/batch/virtual_project/project.rs";
const materializeFile = "crates/vize_canon/src/batch/virtual_project/materialize.rs";
const observerFile =
  "crates/vize_canon/src/batch/virtual_project/project/owned_read_buffer_observation.rs";
const baseDirectory = "crates/vize_canon/src/batch/virtual_project/";
const otherFiles = {
  build: "build.rs",
  script: "build/script_build.rs",
  jsx: "jsx_build.rs",
  artifacts: "project/artifacts.rs",
} as const;
type Builders = Record<keyof typeof otherFiles, string>;
export const readRoutes = [
  "register_path",
  "register_canonical_paths_lenient",
  "register_paths",
  "register_ts_file",
] as const;

function once(source: string, before: string, after: string) {
  assert(before.length > 0 && source.split(before).length === 2, "unique exact source seam");
  return source.replace(before, after);
}

export function instrumentReadSources(project: string, materialize: string, builders: Builders) {
  assert(!project.includes("owned_read_buffer_observation"));
  let observed = once(
    project,
    "mod package_routes;",
    'mod package_routes;\n#[cfg(feature = "profiling")]\npub(super) mod owned_read_buffer_observation;',
  );
  for (const route of readRoutes) {
    const start = observed.indexOf(`fn ${route}(`);
    assert(start >= 0);
    const next = observed.indexOf("\n    ///", start);
    const end = next < 0 ? observed.length : next;
    const body = observed.slice(start, end);
    const declaration =
      route === "register_ts_file"
        ? "let content = std::fs::read_to_string(path)?;"
        : 'let content = profile!("canon.file.read", std::fs::read_to_string(path))?;';
    const replacement =
      declaration +
      `\n                #[cfg(feature = "profiling")]\n                owned_read_buffer_observation::read(path, &content, "${route}");`;
    observed =
      observed.slice(0, start) + once(body, declaration, replacement) + observed.slice(end);
  }
  const entry = "    pub fn materialize(&self) -> CorsaResult<()> {";
  const snapshot =
    entry +
    '\n        #[cfg(feature = "profiling")]\n        super::project::owned_read_buffer_observation::final_project(self);';
  const output = once(materialize, entry, snapshot);
  // Whole inverse: no silently selected source subset or incidental rewrite.
  let inverse = observed;
  for (const route of readRoutes)
    inverse = once(
      inverse,
      `\n                #[cfg(feature = "profiling")]\n                owned_read_buffer_observation::read(path, &content, "${route}");`,
      "",
    );
  inverse = once(
    inverse,
    '\n#[cfg(feature = "profiling")]\npub(super) mod owned_read_buffer_observation;',
    "",
  );
  assert.equal(inverse, project);
  assert.equal(once(output, snapshot, entry), materialize);
  const patched = { ...builders };
  const observerPath = "crate::batch::virtual_project::project::owned_read_buffer_observation";
  patched.build = once(
    patched.build,
    "pub(super) struct RegisteredFile {",
    'pub(super) struct RegisteredFile {\n    #[cfg(feature = "profiling")]\n    pub read_observation: Option<u64>,',
  );
  for (const [key, name] of [
    ["build", "build_vue_registered_file"],
    ["script", "build_script_registered_file"],
    ["jsx", "build_jsx_registered_file"],
  ] as const) {
    const source = patched[key],
      start = source.indexOf(`fn ${name}(`);
    assert(start >= 0);
    const headerEnd =
      source.indexOf(") -> CorsaResult<RegisteredFile> {", start) +
      ") -> CorsaResult<RegisteredFile> {".length;
    const constructor = source.indexOf("    Ok(RegisteredFile {", headerEnd);
    const end = source.indexOf("    })\n}", constructor) + "    })\n}".length;
    assert(headerEnd > start && constructor > headerEnd && end > constructor);
    const original = source.slice(start, end),
      header = source.slice(start, headerEnd);
    const owned = header.includes("Cow<");
    const input = owned ? "source.as_ref()" : key === "build" ? "authored_content" : "content";
    const begin = `\n    #[cfg(feature = "profiling")]\n    let __read_observation = ${observerPath}::begin(path, ${input}, ${owned ? "matches!(&source, Cow::Owned(_))" : "false"});`;
    let replacement = source.slice(start, headerEnd) + begin + source.slice(headerEnd, end);
    replacement = once(
      replacement,
      "    Ok(RegisteredFile {",
      '    let mut __observed_registered = RegisteredFile {\n        #[cfg(feature = "profiling")]\n        read_observation: None,',
    );
    replacement = once(
      replacement,
      "    })\n}",
      `    };\n    #[cfg(feature = "profiling")]\n    { __observed_registered.read_observation = __read_observation.success(&__observed_registered.original_content); }\n    Ok(__observed_registered)\n}`,
    );
    patched[key] = once(source, original, replacement);
  }
  patched.artifacts = once(
    patched.artifacts,
    "        self.reconcile_shared_helper_membership(shared_helpers_before);",
    "        self.reconcile_shared_helper_membership(shared_helpers_before);",
  );
  const success =
    "        if auto_import_count != self.virtual_ts_options.auto_import_stubs.len() {\n            self.mark_incremental_stub_files();\n        }\n        Ok(())";
  patched.artifacts = once(
    patched.artifacts,
    success,
    success.replace(
      "        Ok(())",
      `        #[cfg(feature = "profiling")]\n        ${observerPath}::registered(registered.read_observation, &canonical_virtual);\n        Ok(())`,
    ),
  );
  return { project: observed, materialize: output, builders: patched };
}

export function prepareReadObservation(options: {
  sourceRoot: string;
  sourceSha: string;
  probeRoot: string;
  out: string;
}) {
  assert(path.isAbsolute(options.probeRoot) && path.isAbsolute(options.out));
  assert(!fs.existsSync(options.probeRoot) && !fs.existsSync(options.out));
  fs.mkdirSync(options.out, { recursive: true });
  const git = (args: string[], root = options.sourceRoot, input?: string) =>
    execFileSync("git", args, { cwd: root, input, encoding: "utf8" }).trim();
  assert.equal(git(["rev-parse", "HEAD"]), options.sourceSha);
  assert.equal(git(["status", "--porcelain", "--untracked-files=all"]), "");
  const before = {
    project: fs.readFileSync(path.join(options.sourceRoot, projectFile)),
    materialize: fs.readFileSync(path.join(options.sourceRoot, materializeFile)),
  };
  const builders = Object.fromEntries(
    Object.entries(otherFiles).map(([name, file]) => [
      name,
      fs.readFileSync(path.join(options.sourceRoot, baseDirectory, file), "utf8"),
    ]),
  ) as Builders;
  const changed = instrumentReadSources(
    before.project.toString(),
    before.materialize.toString(),
    builders,
  );
  const observer = fs.readFileSync(
    new URL("./cold-typechecker-scale-read-buffer.rs.txt", import.meta.url),
  );
  git(["worktree", "add", "--detach", options.probeRoot, options.sourceSha]);
  for (const [file, body] of [
    [projectFile, changed.project],
    [materializeFile, changed.materialize],
    [observerFile, observer],
  ] as const)
    fs.writeFileSync(path.join(options.probeRoot, file), body);
  for (const [name, file] of Object.entries(otherFiles))
    fs.writeFileSync(
      path.join(options.probeRoot, baseDirectory, file),
      changed.builders[name as keyof Builders],
    );
  git(
    [
      "add",
      "--",
      projectFile,
      materializeFile,
      observerFile,
      ...Object.values(otherFiles).map((file) => baseDirectory + file),
    ],
    options.probeRoot,
  );
  const tree = git(["write-tree"], options.probeRoot);
  const commit = git(
    [
      "-c",
      "user.name=Benchmark source observer",
      "-c",
      "user.email=benchmark@invalid.example",
      "commit-tree",
      tree,
      "-p",
      options.sourceSha,
    ],
    options.probeRoot,
    "test(bench): observe owned filesystem buffers\n",
  );
  git(["update-ref", "HEAD", commit, options.sourceSha], options.probeRoot);
  const patch = execFileSync("git", ["diff", "--binary", options.sourceSha, commit], {
    cwd: options.probeRoot,
  });
  fs.writeFileSync(path.join(options.out, "instrumentation.patch"), patch, { flag: "wx" });
  const receipt = {
    productSource: options.sourceSha,
    instrumentedSource: commit,
    tree,
    sourceRoot: options.sourceRoot,
    probeRoot: options.probeRoot,
    before: { project: hash(before.project), materialize: hash(before.materialize) },
    observerSha256: hash(observer),
    patchSha256: hash(patch),
    normalImageModified: false,
    instrumentationOnly: true,
  };
  fs.writeFileSync(path.join(options.out, "source.json"), JSON.stringify(receipt, null, 2) + "\n", {
    flag: "wx",
  });
  const build = buildColdScaleBinary({
    sourceRoot: options.probeRoot,
    targetDir: path.join(options.out, "target"),
    binary: path.join(options.out, "binary", "vize"),
    receipt: path.join(options.out, "build.json"),
    role: "instrumented-read-buffer",
    allocation: true,
  });
  return { receipt, build };
}

type Read = {
  kind: string;
  id: number;
  path: string;
  canonical: string;
  route: string;
  length: number;
  capacity: number;
  pointer: string;
  sha256: string;
};
type Final = {
  kind: string;
  projectRoot: string;
  virtualRoot: string;
  rows: {
    original: string;
    virtualPath: string;
    read: Read | null;
    retainedLength: number;
    retainedCapacity: number;
    retainedPointer: string;
    retainedSha256: string;
  }[];
};

export function readBufferDistribution(text: string) {
  assert(text.endsWith("\n"), "complete newline-terminated trace");
  const values = text
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line) as Record<string, unknown>);
  const reads: Read[] = [],
    snapshots: Final[] = [];
  const builds = new Map<number, Record<string, unknown>>(),
    registrations = new Map<string, number | null>(),
    aborted: number[] = [];
  for (const value of values) {
    if (value.kind === "read") {
      const row = value as unknown as Read;
      assert.deepEqual(
        Object.keys(row).sort(),
        [
          "kind",
          "id",
          "path",
          "canonical",
          "route",
          "length",
          "capacity",
          "pointer",
          "sha256",
        ].sort(),
      );
      assert.equal(row.id, reads.length);
      assert(readRoutes.includes(row.route as (typeof readRoutes)[number]));
      for (const n of [row.length, row.capacity]) assert(Number.isSafeInteger(n) && n >= 0);
      assert(
        row.capacity >= row.length && path.isAbsolute(row.path) && path.isAbsolute(row.canonical),
      );
      assert.match(row.pointer, /^0x[0-9a-f]+$/u);
      assert.match(row.sha256, /^[a-f0-9]{64}$/u);
      reads.push(row);
    } else if (value.kind === "built") {
      assert.deepEqual(
        Object.keys(value).sort(),
        ["kind", "id", "inputOwned", "length", "capacity", "pointer", "sha256"].sort(),
      );
      const id = Number(value.id);
      assert(Number.isSafeInteger(id) && reads[id]);
      assert(!builds.has(id) && !aborted.includes(id));
      assert.equal(typeof value.inputOwned, "boolean");
      assert.equal(value.length, reads[id].length);
      assert.equal(value.sha256, reads[id].sha256);
      assert(
        Number.isSafeInteger(value.capacity) && Number(value.capacity) >= Number(value.length),
      );
      assert.match(String(value.pointer), /^0x[0-9a-f]+$/u);
      if (value.inputOwned === false)
        assert.notEqual(
          value.pointer,
          reads[id].pointer,
          "borrowed read input is alive during the builder copy control",
        );
      builds.set(id, value);
    } else if (value.kind === "build-aborted") {
      assert.deepEqual(Object.keys(value).sort(), ["kind", "id"].sort());
      const id = Number(value.id);
      assert(reads[id] && !builds.has(id) && !aborted.includes(id));
      aborted.push(id);
    } else if (value.kind === "registered") {
      assert.deepEqual(Object.keys(value).sort(), ["kind", "id", "virtualPath"].sort());
      assert(typeof value.virtualPath === "string" && path.isAbsolute(value.virtualPath));
      assert(value.id === null || (Number.isSafeInteger(value.id) && builds.has(Number(value.id))));
      registrations.set(value.virtualPath, value.id === null ? null : Number(value.id));
    } else {
      assert.equal(value.kind, "final-project");
      const snapshot = value as unknown as Final;
      assert.deepEqual(
        Object.keys(snapshot).sort(),
        ["kind", "projectRoot", "virtualRoot", "rows"].sort(),
      );
      assert(Array.isArray(snapshot.rows) && snapshot.rows.length > 0);
      const seen = new Set<string>();
      for (const row of snapshot.rows) {
        assert.deepEqual(
          Object.keys(row).sort(),
          [
            "original",
            "virtualPath",
            "read",
            "retainedLength",
            "retainedCapacity",
            "retainedPointer",
            "retainedSha256",
          ].sort(),
        );
        assert(!seen.has(row.virtualPath));
        seen.add(row.virtualPath);
        assert.equal(
          row.read?.id ?? null,
          registrations.get(row.virtualPath) ?? null,
          "actual last registered call, never latest-read guessing",
        );
      }
      snapshots.push(snapshot);
    }
  }
  assert(reads.length > 0 && snapshots.length > 0);
  const final = snapshots.at(-1)!;
  assert.deepEqual(
    Object.keys(final).sort(),
    ["kind", "projectRoot", "virtualRoot", "rows"].sort(),
  );
  assert(path.isAbsolute(final.projectRoot) && path.isAbsolute(final.virtualRoot));
  const originals = new Set<string>(),
    virtuals = new Set<string>();
  const rows = final.rows.map((row) => {
    assert.deepEqual(
      Object.keys(row).sort(),
      [
        "original",
        "virtualPath",
        "read",
        "retainedLength",
        "retainedCapacity",
        "retainedPointer",
        "retainedSha256",
      ].sort(),
    );
    assert(!originals.has(row.original) && !virtuals.has(row.virtualPath));
    originals.add(row.original);
    virtuals.add(row.virtualPath);
    for (const n of [row.retainedLength, row.retainedCapacity])
      assert(Number.isSafeInteger(n) && n >= 0);
    assert(row.retainedCapacity >= row.retainedLength);
    assert.match(row.retainedPointer, /^0x[0-9a-f]+$/u);
    assert.match(row.retainedSha256, /^[a-f0-9]{64}$/u);
    if (row.read === null) return { ...row, copyBytes: null, sameReadAllocation: null };
    assert.deepEqual(row.read, reads[row.read.id], "whole reference to an actual earlier read");
    assert.equal(row.retainedLength, row.read.length);
    assert.equal(row.retainedSha256, row.read.sha256);
    const built = builds.get(row.read.id);
    assert(built);
    const same =
      built.inputOwned === true &&
      row.retainedPointer === row.read.pointer &&
      built.pointer === row.read.pointer;
    return { ...row, built, copyBytes: same ? 0 : row.read.length, sameReadAllocation: same };
  });
  const joined = new Set(rows.flatMap((row) => (row.read === null ? [] : [row.read.id])));
  return {
    reads,
    abortedReads: aborted,
    successfulBuilds: [...builds.values()],
    registeredCalls: [...registrations],
    unjoinedReadIds: reads.filter((row) => !joined.has(row.id)).map((row) => row.id),
    snapshots: snapshots.length,
    rows,
    actualReadBytes: reads.reduce((n, r) => n + r.length, 0),
    readCapacityBytes: reads.reduce((n, r) => n + r.capacity, 0),
    retainedBytes: rows.reduce((n, r) => n + r.retainedLength, 0),
    retainedCapacityBytes: rows.reduce((n, r) => n + r.retainedCapacity, 0),
    joinedCopyBytes: rows.reduce((n, r) => n + (r.copyBytes ?? 0), 0),
    borrowedOrUnjoinedFinalRows: rows.filter((r) => r.read === null).length,
    scope:
      "all raw read events and last materialization's actual successful registered source owners; unjoined/failed/superseded reads get no saving credit; instrumented-only, not primary RSS/latency",
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const [sourceRoot, sourceSha, probeRoot, out] = process.argv.slice(2);
  assert(sourceRoot && sourceSha && probeRoot && out);
  prepareReadObservation({ sourceRoot, sourceSha, probeRoot, out });
}
