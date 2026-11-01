import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, posix } from "node:path";
import test from "node:test";
import { readTarGz } from "../../../tools/support/archive/tar.ts";
import { fileImage } from "../../../tools/support/release/computed_event_custody.ts";
import { capturePublicPayload } from "../../../tools/support/release/human_reports_11252_11253_public/capture-payload.ts";
import { collectRootCommandCensus } from "../../../tools/support/release/human_reports_11252_11253_public/command-census.ts";
import { captureRuntimePaths } from "../../../tools/support/release/human_reports_11252_11253_public/runtime-chain.ts";
import { humanDependencyBinPartition } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-partition.ts";
import type { FileFacts } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-types.ts";
import {
  binPolicyPath,
  bindBinPolicy,
} from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-policy.ts";
import {
  derivedNodePath,
  renderPnpm12161Unix,
} from "../../../tools/support/release/human_reports_11252_11253_public/pnpm-shim-render.ts";
import {
  gitBlob,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import { expectedNode, fixture, pkg } from "./human-reports-11252-11253-bin.fixture.ts";

// All packages, native-looking bytes and runtimes below are synthetic inert
// filesystem inputs. No executable, shell, package manager or native is run.
function setup() {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "human-command-census-"))),
    root = join(base, "consumer-original");
  const directory = join(root, "node_modules/typescript");
  const parent = pkg(
    "typescript",
    "5.9.3",
    directory,
    { name: "typescript", version: "5.9.3", bin: { tsc: "bin/tsc", tsserver: "bin/tsserver" } },
    {
      "bin/tsc": Buffer.from("#!/usr/bin/env node\n// synthetic compiler; never executed\n"),
      "bin/tsserver": Buffer.from("#!/usr/bin/env node\n// synthetic server; never executed\n"),
    },
  );
  assert(parent.archive);
  for (const member of readTarGz(parent.archive)) {
    const path = join(directory, member.name.slice(8));
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, member.data);
  }
  const manifest = Buffer.from('{"name":"synthetic","devDependencies":{"typescript":"5.9.3"}}\n');
  writeFileSync(join(root, "package.json"), manifest);
  const bin = join(root, "node_modules/.bin");
  mkdirSync(bin);
  for (const name of ["tsc", "tsserver"]) {
    const path = join(bin, name),
      body = renderPnpm12161Unix({
        target: join(directory, "bin", name),
        shimPath: path,
        runtime: "node",
        nodePath: derivedNodePath(directory, [join(root, "node_modules/.pnpm/node_modules")]),
        relocatableRoot: root,
      });
    writeFileSync(path, body);
    chmodSync(path, 0o755);
  }
  const raw = readFileSync(
    new URL(
      "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-policy.json.txt",
      import.meta.url,
    ),
  );
  const policy = bindBinPolicy(raw, {
    head: "1".repeat(40),
    files: [{ path: binPolicyPath, bytes: raw.length, sha256: sha256(raw), gitBlob: gitBlob(raw) }],
  });
  const payload = capturePublicPayload(parent, () => {});
  const events: unknown[] = [],
    sink = (kind: string, value: unknown) => events.push({ kind, value });
  return {
    base,
    root,
    directory,
    manifest,
    bin,
    parent,
    payload,
    policy,
    events,
    sink,
    close: () => rmSync(base, { recursive: true, force: true }),
  };
}
test("complete root alias/bin population derives whole wrappers and public targets without writer credit", () => {
  const f = setup();
  try {
    const result = collectRootCommandCensus(f.root, f.manifest, [f.payload], f.policy, f.sink);
    assert.deepEqual(
      result.aliases.map((r) => r.name),
      ["typescript"],
    );
    assert.deepEqual(
      result.targets.map((r) => r.name),
      ["tsc", "tsserver"],
    );
    assert(
      result.targets.every(
        (r) => r.ownerDirectory === f.directory && r.wrapper.rawBase64.length > 0,
      ),
    );
    assert.equal(result.actualWriterRuntimeAttribution, false);
  } finally {
    f.close();
  }
});
test("an unknown root executable refuses before its body can enter evidence", () => {
  const f = setup(),
    secret = "UNEXPECTED_ROOT_EXEC_TOKEN_812";
  try {
    writeFileSync(join(f.bin, "sh"), secret);
    chmodSync(join(f.bin, "sh"), 0o755);
    assert.throws(
      () => collectRootCommandCensus(f.root, f.manifest, [f.payload], f.policy, f.sink),
      /unknown\/missing root executable/u,
    );
    assert(!JSON.stringify(f.events).includes(secret));
    assert(!JSON.stringify(f.events).includes(Buffer.from(secret).toString("base64")));
  } finally {
    f.close();
  }
});
test("changed whole root wrapper, redirect and unlisted root owner remain mandatory refusals", () => {
  for (const mutation of ["body", "redirect", "owner"] as const) {
    const f = setup();
    try {
      if (mutation === "body")
        writeFileSync(join(f.bin, "tsc"), "#!/bin/sh\nUNEXPECTED_WRAPPER_SECRET_391\n");
      if (mutation === "redirect") {
        rmSync(join(f.bin, "tsc"));
        symlinkSync(join(f.directory, "bin/tsc"), join(f.bin, "tsc"));
      }
      if (mutation === "owner") mkdirSync(join(f.root, "node_modules/unlisted"));
      assert.throws(() =>
        collectRootCommandCensus(f.root, f.manifest, [f.payload], f.policy, f.sink),
      );
      assert(!JSON.stringify(f.events).includes("UNEXPECTED_WRAPPER_SECRET_391"));
    } finally {
      f.close();
    }
  }
});
test("a second authentic candidate cannot be silently sorted into an executable winner", () => {
  const f = setup();
  try {
    const directory = join(f.root, "node_modules/another"),
      other = pkg(
        "another",
        "1.0.0",
        directory,
        { name: "another", version: "1.0.0", bin: { tsc: "bin/tsc" } },
        { "bin/tsc": Buffer.from("#!/usr/bin/env node\n// inert alternate\n") },
      );
    assert(other.archive);
    for (const member of readTarGz(other.archive)) {
      const path = join(directory, member.name.slice(8));
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, member.data);
    }
    const payload = capturePublicPayload(other, () => {});
    assert.throws(
      () => collectRootCommandCensus(f.root, f.manifest, [f.payload, payload], f.policy, f.sink),
      /owner\/winner is ambiguous/u,
    );
  } finally {
    f.close();
  }
});
test("source self placement requires its own complete payload/roster and never invents a dependency edge", () => {
  const source = fixture(),
    owner = source.owner;
  assert(owner.archive);
  const inventory = new Map<string, FileFacts>();
  for (const member of readTarGz(owner.archive)) {
    const path = posix.join(owner.directory, member.name.slice(8));
    inventory.set(path, { path, realPath: path, kind: "file", body: member.data });
  }
  for (const name of ["tsc", "tsserver"]) {
    const path = posix.join(owner.directory, "node_modules/.bin", name);
    inventory.set(path, {
      path,
      realPath: path,
      kind: "file",
      body: expectedNode(name, true, true),
    });
  }
  const files = [...inventory.keys()];
  for (const file of files) {
    let path = posix.dirname(file);
    while (path !== owner.directory) {
      inventory.set(path, { path, realPath: path, kind: "directory", body: null });
      path = posix.dirname(path);
    }
  }
  const facts = {
    ...source,
    parent: owner,
    dependencyOwners: [],
    graph: [],
    inventory: [...inventory.values()],
  };
  const proof = humanDependencyBinPartition(facts);
  assert.equal(proof.publicPayloadCount, 3);
  assert.equal(proof.sourceConformantBinCount, 2);
  assert.equal(proof.accepted, false);
  assert.equal(proof.actualWriterRuntimeAttribution, false);
  assert.throws(() =>
    humanDependencyBinPartition({
      ...facts,
      graph: [
        {
          parentDirectory: owner.directory,
          dependencyName: "typescript",
          version: owner.version,
          directory: owner.directory,
        },
      ],
    }),
  );
  assert.throws(() =>
    humanDependencyBinPartition({
      ...facts,
      inventory: facts.inventory.filter((r) => !r.path.endsWith("/.bin/tsc")),
    }),
  );
});
function runtime() {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "human-runtime-path-"))),
    root = join(base, "consumer-original"),
    bin = join(base, "runtime-bin");
  mkdirSync(join(root, "node_modules/.bin"), { recursive: true });
  mkdirSync(bin);
  const node = join(bin, "node"),
    manager = join(base, "manager-native-inert");
  writeFileSync(node, "synthetic runtime image, never executed\n");
  chmodSync(node, 0o755);
  const bytes = Buffer.alloc(4201);
  Buffer.from("7f454c46", "hex").copy(bytes);
  writeFileSync(manager, bytes);
  chmodSync(manager, 0o755);
  const env = { PATH: bin + ":/bin:/usr/bin" },
    events: unknown[] = [],
    sink = (kind: string, value: unknown) => events.push({ kind, value });
  return {
    base,
    root,
    bin,
    node: fileImage(node),
    manager: fileImage(manager),
    env,
    events,
    sink,
    close: () => rmSync(base, { recursive: true, force: true }),
  };
}
test("source PATH model preserves prefix duplicates and identifies Node plus OS shell without executing them", () => {
  const f = runtime();
  try {
    const own = join(f.root, "node_modules/.bin");
    const result = captureRuntimePaths(f.root, f.manager, f.node, f.env, [own], f.sink);
    assert.deepEqual(result.execPaths.slice(0, 2), [own, own]);
    assert.deepEqual(result.scriptPaths.slice(0, 2), [own, own]);
    assert.equal(result.bundledNodeGyp, null);
    assert.equal(result.managerWriterImageOwner, "UNKNOWN");
    assert(
      result.selected
        .filter((r) => r.command === "node")
        .every((r) => r.image.sha256 === f.node.sha256),
    );
    assert(
      result.selected
        .filter((r) => r.command === "sh")
        .every((r) => r.image.sha256 === fileImage("/bin/sh").sha256),
    );
    assert.deepEqual(f.env, { PATH: f.bin + ":/bin:/usr/bin" });
  } finally {
    f.close();
  }
});
test("root runtime shadows and earlier PATH Node/shell substitutes refuse without executing or exporting bytes", () => {
  for (const mutation of ["local-node", "path-node", "path-sh"] as const) {
    const f = runtime(),
      secret = "UNEXPECTED_RUNTIME_SECRET_991";
    try {
      const directory =
        mutation === "local-node" ? join(f.root, "node_modules/.bin") : join(f.base, "earlier");
      mkdirSync(directory, { recursive: true });
      const path = join(directory, mutation === "path-sh" ? "sh" : "node");
      writeFileSync(path, secret);
      chmodSync(path, 0o755);
      const env = mutation === "local-node" ? f.env : { PATH: directory + ":" + f.env.PATH };
      assert.throws(() => captureRuntimePaths(f.root, f.manager, f.node, env, [], f.sink));
      assert(!JSON.stringify(f.events).includes(secret));
      assert(!JSON.stringify(f.events).includes(Buffer.from(secret).toString("base64")));
    } finally {
      f.close();
    }
  }
});
