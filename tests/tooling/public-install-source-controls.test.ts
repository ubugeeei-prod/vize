// Raw source-object and genuine committed manifest-shape rejection laws.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  collect,
  type CollectArguments,
} from "../../tools/support/release/public_install/collect.ts";
import {
  overrides,
  producerAuthority,
  sourceFile,
  sourceIdentity,
  sourceManifests,
} from "../../tools/support/release/public_install/identity.ts";

const repository = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
for (const key of [...overrides, "VIZE_PUBLIC_NATIVE_CUSTODY", "VIZE_OXLINT_NATIVE_CUSTODY"])
  delete process.env[key];
globalThis.fetch = () => {
  throw new Error("inert controls must not reach the network");
};

type Manifest = Record<string, unknown> & {
  version: string;
  dependencies: Record<string, string>;
  optionalDependencies: Record<string, string>;
};
const manifestPaths = [
  "npm/cli/package.json",
  "npm/oxlint/package.json",
  "npm/native/package.json",
];
const actualHead = execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
  cwd: repository,
  encoding: "utf8",
}).trim();
const original = new Map(
  manifestPaths.map((name) => [
    name,
    JSON.parse(sourceFile(repository, actualHead, name).toString("utf8")) as Manifest,
  ]),
);
const cliManifest = (): Manifest =>
  structuredClone(original.get("npm/cli/package.json") as Manifest);
const workspaceText = sourceFile(repository, actualHead, "pnpm-workspace.yaml").toString("utf8");
const wrapper = sourceFile(repository, actualHead, "npm/cli/bin/vize");

function fixture(t: { after: (fn: () => void) => void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "public-install-source-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const git = (...args: string[]): string =>
    execFileSync(
      "git",
      ["-c", "user.name=Inert test", "-c", "user.email=inert@example.invalid", ...args],
      { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    ).trim();
  const writeShape = (version: string): void => {
    for (const [name, manifest] of original) {
      fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
      fs.writeFileSync(
        path.join(root, name),
        JSON.stringify({ ...structuredClone(manifest), version }),
      );
    }
    const workspace = workspaceText.replace(
      /^( {4}"@vizejs\/native-[^"]+": ")[^"]+(".*)$/gmu,
      (_, start: string, end: string) => `${start}${version}${end}`,
    );
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), workspace);
    fs.mkdirSync(path.join(root, "npm/cli/bin"), { recursive: true });
    fs.writeFileSync(path.join(root, "npm/cli/bin/vize"), wrapper);
  };
  const commit = (): string => {
    git("add", ".");
    git("commit", "-qm", "inert committed manifest shape");
    return git("rev-parse", "HEAD");
  };
  git("init", "-q");
  git("commit", "--allow-empty", "-qm", "inert C");
  const cut = git("rev-parse", "HEAD");
  writeShape("0.999.0");
  const fakeNode = (): { node: string; sentinel: string } => {
    const sentinel = path.join(root, "node-started");
    const node = path.join(root, "fake-node");
    fs.writeFileSync(node, `#!/bin/sh\ntouch '${sentinel}'\nexit 99\n`, { mode: 0o700 });
    return { node, sentinel };
  };
  const args = (head: string, tag: string, install: string, node: string): CollectArguments => ({
    root,
    cut,
    head,
    tag,
    run: "123",
    "install-root": install,
    node,
    output: path.join(root, "receipt.json"),
    "collector-sha256": producerAuthority(repository).sha256,
  });
  return { root, git, writeShape, commit, cut, fakeNode, args };
}

const source = (head: string, version = "0.999.0") => ({ H: head, tag: `v${version}` });

describe("SourceControls", () => {
  test("genuine committed CLI, workspace and Corsa shape is retained", (t) => {
    const f = fixture(t);
    const version = cliManifest().version;
    const receipt = sourceManifests(repository, source(actualHead, version));
    assert.equal(receipt.files.length, 4);
    assert.ok(Number(receipt.corsaVersion.split(".")[0]) >= 7);
    assert.equal("@vizejs/native-darwin-arm64" in cliManifest().optionalDependencies, false);
    assert.equal(sourceManifests(f.root, source(f.commit())).files.length, 4);
  });

  test("wrong H versions, native edges, platform and Corsa refuse", (t) => {
    const f = fixture(t);
    const mutations: readonly [string, string, string | null, string][] = [
      ["npm/cli/package.json", "version", null, "0.998.0"],
      ["npm/native/package.json", "version", null, "0.998.0"],
      ["npm/cli/package.json", "dependencies", "@vizejs/native", "workspace:^"],
      ["npm/cli/package.json", "dependencies", "@vizejs/native", "0.998.0"],
      [
        "npm/cli/package.json",
        "optionalDependencies",
        "@typescript/typescript-darwin-arm64",
        "catalog:wrong",
      ],
      ["npm/oxlint/package.json", "optionalDependencies", "@vizejs/native-darwin-arm64", "0.998.0"],
    ];
    for (const [name, field, child, value] of mutations) {
      f.writeShape("0.999.0");
      const file = path.join(f.root, name);
      const manifest = JSON.parse(fs.readFileSync(file, "utf8")) as Record<string, unknown>;
      if (child === null) manifest[field] = value;
      else (manifest[field] as Record<string, string>)[child] = value;
      fs.writeFileSync(file, JSON.stringify(manifest));
      assert.throws(() => sourceManifests(f.root, source(f.commit())), `${name} ${field}`);
    }
    const corsaVersion = sourceManifests(
      repository,
      source(actualHead, cliManifest().version),
    ).corsaVersion;
    for (const [before, after] of [
      ["0.999.0", "0.998.0"],
      [`"${corsaVersion}"`, '"6.9.0"'],
    ] as const) {
      f.writeShape("0.999.0");
      const workspace = path.join(f.root, "pnpm-workspace.yaml");
      assert.ok(fs.readFileSync(workspace, "utf8").includes(before));
      fs.writeFileSync(workspace, fs.readFileSync(workspace, "utf8").replaceAll(before, after));
      assert.throws(() => sourceManifests(f.root, source(f.commit())));
    }
  });

  test("full identity and exact raw single parent are required", (t) => {
    const f = fixture(t);
    const head = f.commit();
    assert.equal(sourceIdentity(f.root, f.cut, head, "v0.999.0", "123").H, head);
    for (const [cut, candidate, tag, run] of [
      [head, f.cut, "v0.999.0", "123"],
      [f.cut.slice(0, 8), head, "v0.999.0", "123"],
      [f.cut, head, "v0.999.1", "123"],
      [f.cut, head, "v0.999.0", "0"],
    ] as const)
      assert.throws(() => sourceIdentity(f.root, cut, candidate, tag, run));
  });

  test("installed CLI native and Corsa must equal actual H edges before probe", async (t) => {
    const f = fixture(t);
    const head = f.commit();
    const corsa = sourceManifests(f.root, source(head)).corsaVersion;
    const install = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "public-install-consumer-")),
    );
    t.after(() => fs.rmSync(install, { recursive: true, force: true }));
    const versions = Object.fromEntries(
      ["vize", "oxlint-plugin-vize", "@vizejs/native", "@vizejs/native-darwin-arm64"].map(
        (name) => [name, "0.999.0"],
      ),
    );
    fs.writeFileSync(
      path.join(install, "package.json"),
      JSON.stringify({ private: true, dependencies: versions }),
    );
    fs.writeFileSync(
      path.join(install, "package-lock.json"),
      JSON.stringify({ lockfileVersion: 3, packages: { "": { dependencies: versions } } }),
    );
    const manifestPath = path.join(install, "node_modules/vize/package.json");
    fs.mkdirSync(path.dirname(manifestPath), { recursive: true });
    const { node, sentinel } = f.fakeNode();
    for (const edge of ["native", "corsa"]) {
      const manifest = cliManifest();
      manifest.dependencies["@vizejs/native"] = edge === "native" ? "0.998.0" : "0.999.0";
      manifest.optionalDependencies["@typescript/typescript-darwin-arm64"] =
        edge === "corsa" ? "7.999.0" : corsa;
      fs.writeFileSync(manifestPath, JSON.stringify(manifest));
      await assert.rejects(
        collect(f.args(head, "v0.999.0", install, node)),
        /actual H declaration/u,
      );
      assert.equal(fs.existsSync(sentinel), false);
      assert.equal(fs.existsSync(path.join(f.root, "receipt.json")), false);
    }
  });

  test("Git replacement cannot substitute H source or CLI wrapper", (t) => {
    const f = fixture(t);
    f.writeShape("0.998.0");
    const head = f.commit();
    assert.throws(() => sourceManifests(f.root, source(head)));
    f.writeShape("0.999.0");
    fs.writeFileSync(path.join(f.root, "npm/cli/bin/vize"), "inert replacement wrapper");
    f.git("add", ".");
    const replacement = f.git(
      "commit-tree",
      f.git("write-tree"),
      "-p",
      f.cut,
      "-m",
      "inert replacement",
    );
    f.git("replace", head, replacement);
    assert.equal(f.git("show", `${head}:npm/cli/bin/vize`), "inert replacement wrapper");
    assert.ok(sourceFile(f.root, head, "npm/cli/bin/vize").equals(wrapper));
    assert.equal(sourceIdentity(f.root, f.cut, head, "v0.999.0", "123").H, head);
    assert.throws(() => sourceManifests(f.root, source(head)));
  });

  test("legacy graft cannot hide a raw second parent", (t) => {
    const f = fixture(t);
    const extra = f.commit();
    const head = f.git(
      "commit-tree",
      f.git("rev-parse", "HEAD^{tree}"),
      "-p",
      f.cut,
      "-p",
      extra,
      "-m",
      "inert two-parent H",
    );
    assert.throws(() => sourceIdentity(f.root, f.cut, head, "v0.999.0", "123"));
    fs.writeFileSync(path.join(f.root, ".git/info/grafts"), `${head} ${f.cut}\n`);
    assert.equal(
      f.git("--no-replace-objects", "rev-list", "--parents", "-n", "1", head),
      `${head} ${f.cut}`,
    );
    assert.throws(() => sourceIdentity(f.root, f.cut, head, "v0.999.0", "123"));
  });

  test("copied committed shape with a wrong version refuses before the fake Node", async (t) => {
    // PR checkouts may be shallow two-parent merge commits. Copy their raw
    // manifest/catalog bytes into owned actual C -> H history for this law.
    // The copied shape is inert control data, never publication authority.
    const f = fixture(t);
    for (const name of [...manifestPaths, "pnpm-workspace.yaml", "npm/cli/bin/vize"]) {
      fs.mkdirSync(path.dirname(path.join(f.root, name)), { recursive: true });
      fs.writeFileSync(path.join(f.root, name), sourceFile(repository, actualHead, name));
    }
    const head = f.commit();
    assert.equal(sourceIdentity(f.root, f.cut, head, "v0.999999.0", "123").H, head);
    const { node, sentinel } = f.fakeNode();
    await assert.rejects(collect(f.args(head, "v0.999999.0", f.root, node)), /catalog version/u);
    assert.equal(fs.existsSync(sentinel), false);
    assert.equal(fs.existsSync(path.join(f.root, "receipt.json")), false);
  });

  test("genuine two-parent checkout is refused before any probe", async (t) => {
    const f = fixture(t);
    const main = f.git("symbolic-ref", "--short", "HEAD");
    f.commit();
    f.git("branch", "inert-side", f.cut);
    f.git("checkout", "-q", "inert-side");
    f.git("commit", "--allow-empty", "-qm", "inert side");
    f.git("checkout", "-q", main);
    f.git("merge", "--no-ff", "inert-side", "-m", "genuine inert checkout merge");
    const head = f.git("rev-parse", "HEAD");
    const raw = f.git("--no-replace-objects", "cat-file", "commit", head);
    const parents = (raw.split("\n\n", 1)[0] ?? "")
      .split("\n")
      .filter((line) => line.startsWith("parent "))
      .map((line) => line.slice(7));
    assert.equal(parents.length, 2);
    const { node, sentinel } = f.fakeNode();
    const args = { ...f.args(head, "v0.999.0", f.root, node), cut: parents[0] ?? "" };
    await assert.rejects(collect(args), /exact single-parent/u);
    assert.equal(fs.existsSync(sentinel), false);
    assert.equal(fs.existsSync(path.join(f.root, "receipt.json")), false);
  });
});
