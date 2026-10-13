import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { authenticateOwnedCorsaAfterContext } from "../../tools/support/compat/github/owned-corsa-clippy-source.mjs";
import { authenticateOwnedCorsaProducerMetadata } from "../../tools/support/dependencies/owned-corsa-producer-metadata.ts";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const oid = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input });
const capsule = fs.readFileSync(
  path.join(repoRoot, "tools/support/dependencies/owned-corsa-producer-metadata.json"),
);
assert.equal(sha(capsule), "ac929651c22fa9b33f57fa443846b5df6f9fee6ba09d5ee32741ca56063a547c");
const baseline = new Map(
  JSON.parse(capsule).entries.map((entry) => [entry.path, Buffer.from(entry.rawBase64, "base64")]),
);
const originalCSS = new Map(
  ["Cargo.toml", "Cargo.lock"].map((file) => [
    file,
    fs.readFileSync(
      path.join(
        repoRoot,
        "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/metadata",
        file,
      ),
    ),
  ]),
);
assert.equal(
  sha(originalCSS.get("Cargo.toml")),
  "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68",
);
assert.equal(
  sha(originalCSS.get("Cargo.lock")),
  "194c3098330cbeb91e441a2f035fe3cfb37650195c6ff04baf88f09a08378684",
);
const kernel = fs.realpathSync(path.join(repoRoot, "tools/commands/release/pr.rs"));
const kernelBefore = fs.readFileSync(kernel);

// Author future inputs independently from parsed exact cells; only the real Rust kernel admits them.
function pair(version) {
  if (version === "0.443.0")
    return new Map([...baseline].map(([file, body]) => [file, Buffer.from(body)]));
  const root = baseline.get("Cargo.toml").toString();
  const parsed = parseToml(root);
  const aliases = Object.entries(parsed.workspace.dependencies).filter(
    ([key, value]) =>
      (key.startsWith("vize_") || value.package?.startsWith("vize_")) &&
      value.version === "=0.443.0",
  );
  let nextRoot = root;
  for (const [key] of aliases) {
    const lines = nextRoot
      .split("\n")
      .filter((line) => line.startsWith(key + " = ") && line.includes('version = "=0.443.0"'));
    assert.equal(lines.length, 1, key);
    const next = lines[0].replace('version = "=0.443.0"', `version = "=${version}"`);
    assert.notEqual(next, lines[0], key);
    nextRoot = nextRoot.replace(lines[0], next);
  }
  const anchor = '[workspace.package]\nversion = "0.443.0"';
  assert.equal(nextRoot.split(anchor).length, 2);
  nextRoot = nextRoot.replace(anchor, `[workspace.package]\nversion = "${version}"`);
  const lock = baseline.get("Cargo.lock").toString();
  const packages = parseToml(lock).package.filter(
    ({ name, version }) =>
      version === "0.443.0" &&
      (name === "vize" ||
        name.startsWith("vize_") ||
        ["davinci_harness", "davinci_test_support", "vize-zed-extension"].includes(name)),
  );
  let nextLock = lock;
  for (const { name } of packages) {
    const before = `[[package]]\nname = "${name}"\nversion = "0.443.0"`;
    assert.equal(nextLock.split(before).length, 2, name);
    nextLock = nextLock.replace(before, `[[package]]\nname = "${name}"\nversion = "${version}"`);
  }
  return new Map([
    ["Cargo.toml", Buffer.from(nextRoot)],
    ["Cargo.lock", Buffer.from(nextLock)],
  ]);
}

function commit(root, files) {
  for (const file of files) {
    const target = path.join(root, file);
    if (!fs.existsSync(target)) {
      git(root, ["update-index", "--force-remove", file]);
      continue;
    }
    const body = fs.readFileSync(target);
    const blob = git(root, ["hash-object", "-w", "--stdin"], body).toString().trim();
    const mode = fs.statSync(target).mode & 0o111 ? "100755" : "100644";
    git(root, ["update-index", "--add", "--cacheinfo", `${mode},${blob},${file}`]);
  }
  const tree = git(root, ["write-tree"]).toString().trim();
  const head = git(root, ["commit-tree", tree, "-m", "Private complete source-context control"])
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/control", head]);
  git(root, ["symbolic-ref", "HEAD", "refs/heads/control"]);
  return { source: head, tree };
}
function withRoot(version, action) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-css-context-control-"));
  try {
    const actualBodies = new Map();
    for (const row of ownedCorsaSource.rows) {
      const file = path.join(repoRoot, row.file);
      const stat = fs.lstatSync(file);
      assert(stat.isFile() && (stat.mode & 0o777) === 0o644, row.file);
      const body = fs.readFileSync(file);
      actualBodies.set(row.file, { mode: "100644", blob: oid(body), body });
    }
    const afterContext = authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) =>
      actualBodies.get(file),
    );
    assert(afterContext, "complete actual237 source context before historical projection");
    const current = pair(version);
    for (const row of ownedCorsaSource.rows) {
      const body = afterContext.historicalAfter(row.file).body;
      assert.equal(body.length, row.after.bytes, row.file);
      assert.equal(sha(body), row.after.sha256, row.file);
      assert.equal(oid(body), row.after.blob, row.file);
      const target = path.join(root, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, current.get(row.file) ?? afterContext.actualAfter(row.file).body, {
        flag: "wx",
        mode: 0o644,
      });
    }
    assert.equal(ownedCorsaSource.rows.length, 237);
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Vize source control"]);
    git(root, ["config", "user.email", "source-control@example.invalid"]);
    const initial = commit(
      root,
      ownedCorsaSource.rows.map(({ file }) => file),
    );
    return action(root, initial);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function authenticate(
  root,
  sources = new Map(
    [...originalCSS].map(([file, body]) => [
      file,
      {
        baseline: Buffer.from(body),
        actualSha256: sha(fs.readFileSync(path.join(root, file))),
      },
    ]),
  ),
) {
  const result = authenticateOwnedCorsaProducerMetadata(root, sources);
  assert.deepEqual(
    fs.readFileSync(kernel),
    kernelBefore,
    "same unchanged real official kernel source",
  );
  return result;
}
function refused(root, name, mutate, expected) {
  mutate(root);
  let error;
  try {
    authenticate(root);
  } catch (caught) {
    error = caught;
  }
  assert(error instanceof Error, `${name}: no forged source qualification`);
  assert.match(error.message, expected);
  return { name, message: error.message, sourceOnly: true, qualification: false };
}

void test("whole current and two coherent future pairs retain six exact aliases and all237 source bodies", () => {
  for (const version of ["0.443.0", "0.448.0", "0.65535.0"])
    withRoot(version, (root, initial) => {
      const result = authenticate(root);
      assert(result);
      assert.equal(result.currentVersion, version);
      assert.equal(result.baselineVersion, "0.443.0");
      assert.equal(result.wholeForwardAndInverse, true);
      assert.equal(result.ownedCorsaSourceContext.wholeCurrentBodies, 237);
      assert.equal(result.ownedCorsaSourceContext.sourceRevision, initial.source);
      assert.equal(result.ownedCorsaSourceContext.runtimeCredit, false);
      assert.equal(result.publicationQualification, false);
      const dependencies = parseToml(fs.readFileSync(path.join(root, "Cargo.toml"), "utf8"))
        .workspace.dependencies;
      for (const name of [
        "corsa",
        "corsa_core",
        "corsa_jsonrpc",
        "corsa_client",
        "corsa_lsp",
        "corsa_orchestrator",
      ])
        assert.deepEqual(dependencies[name], {
          package: `vize_${name}`,
          version: `=${version}`,
          path: `vendor/${name}`,
        });
      assert.equal(
        parseToml(fs.readFileSync(path.join(root, "Cargo.lock"), "utf8")).package.find(
          ({ name }) => name === "corsa_runtime",
        ).version,
        "1.14.0",
      );
      assert.deepEqual(
        result.ownedCorsaSourceContext.versionTransform.files,
        ["Cargo.toml", "Cargo.lock"].map((file) => ({
          path: file,
          changed: version !== "0.443.0",
        })),
      );
    });
});

void test("real official kernel refuses mixed, stale and unrelated root-lock changes before projection", () => {
  const records = [];
  for (const [name, file, body] of [
    ["mixed-version", "Cargo.lock", pair("0.65535.0").get("Cargo.lock")],
    ["stale-lock", "Cargo.lock", baseline.get("Cargo.lock")],
    [
      "unrelated-root",
      "Cargo.toml",
      Buffer.concat([pair("0.448.0").get("Cargo.toml"), Buffer.from("# unknown producer byte\n")]),
    ],
    [
      "unrelated-lock",
      "Cargo.lock",
      Buffer.concat([pair("0.448.0").get("Cargo.lock"), Buffer.from("# unknown lock byte\n")]),
    ],
  ])
    withRoot("0.448.0", (root) =>
      records.push(
        refused(
          root,
          name,
          () => {
            fs.writeFileSync(path.join(root, file), body);
            commit(root, [file]);
          },
          new RegExp(
            `${file.replace(".", "\\.")} is not the complete reversible official metadata rewrite\\.`,
          ),
        ),
      ),
    );
  assert.equal(records.length, 4);
  console.log(JSON.stringify({ scope: "real metadata kernel refusal only", records }));
});

void test("valid version metadata cannot lend source authority to a changed provider, mode, absence or replacement", () => {
  const file = "vendor/corsa_core/src/lib.rs";
  const records = [];
  for (const name of ["provider-byte", "provider-mode", "provider-missing", "replacement"])
    withRoot("0.448.0", (root, initial) => {
      records.push(
        refused(
          root,
          name,
          () => {
            const target = path.join(root, file);
            if (name === "provider-byte")
              fs.appendFileSync(target, "\n// unreviewed provider byte\n");
            if (name === "provider-mode") fs.chmodSync(target, 0o755);
            if (name === "provider-missing") fs.unlinkSync(target);
            const after = commit(root, [file]);
            if (name === "replacement") {
              const foreignBody = Buffer.concat([
                fs.readFileSync(target),
                Buffer.from("\n// replacement-only foreign body\n"),
              ]);
              const foreignBlob = git(root, ["hash-object", "-w", "--stdin"], foreignBody)
                .toString()
                .trim();
              git(root, ["update-index", "--cacheinfo", `100644,${foreignBlob},${file}`]);
              const foreignTree = git(root, ["write-tree"]).toString().trim();
              const foreignCommit = git(root, [
                "commit-tree",
                foreignTree,
                "-m",
                "Foreign replacement source",
              ])
                .toString()
                .trim();
              assert.notEqual(foreignCommit, initial.source);
              git(root, ["update-ref", `refs/replace/${after.source}`, foreignCommit]);
            }
          },
          name === "replacement" ? /!== 0/u : /corsa_core\/src\/lib\.rs/u,
        ),
      );
    });
  assert.equal(records.length, 4);
  console.log(JSON.stringify({ scope: "actual private237 provider refusal only", records }));
});

void test("an authored fake CSS before buffer cannot replace either complete historical authority", () => {
  withRoot("0.448.0", (root) => {
    for (const changed of ["Cargo.toml", "Cargo.lock"]) {
      const sources = new Map(
        [...originalCSS].map(([file, body]) => [
          file,
          {
            baseline: Buffer.from(file === changed ? baseline.get(file) : body),
            actualSha256: sha(fs.readFileSync(path.join(root, file))),
          },
        ]),
      );
      assert.throws(() => authenticate(root, sources), /Expected values to be strictly equal/u);
    }
  });
});
