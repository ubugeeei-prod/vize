import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

const repository = fileURLToPath(new URL("../..", import.meta.url));
const attributes = readFileSync(path.join(repository, ".gitattributes"), "utf8");
const patches = readdirSync(path.join(repository, "patches")).filter((name) =>
  name.endsWith(".patch"),
);
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function git(root: string, args: string[]) {
  const result = spawnSync("git", args, {
    cwd: root,
    env: {
      ...process.env,
      GIT_CONFIG_NOSYSTEM: "1",
      GIT_CONFIG_GLOBAL: path.join(root, ".empty-global-config"),
    },
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr.toString());
  return result.stdout;
}

function checkout(t: TestContext, autocrlf: string, checkoutAttributes: string) {
  const root = mkdtempSync(path.join(tmpdir(), "vize-release-patch-checkout-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, "patches"));
  writeFileSync(path.join(root, ".gitattributes"), checkoutAttributes);
  git(root, ["init", "--quiet"]);
  git(root, ["config", "core.autocrlf", "false"]);
  git(root, ["config", "core.attributesFile", path.join(root, ".empty-attributes")]);
  const sources = new Map<string, Buffer>();
  for (const name of patches) {
    // Read the committed source, independently of the host checkout's EOL policy.
    const source = git(repository, ["show", `HEAD:patches/${name}`]);
    assert.equal(source.includes(13), false, `${name}: source patch must use LF`);
    sources.set(name, source);
    writeFileSync(path.join(root, "patches", name), source);
  }
  git(root, ["add", ".gitattributes", "patches"]);
  git(root, ["config", "core.autocrlf", autocrlf]);
  for (const name of patches) rmSync(path.join(root, "patches", name));
  git(root, ["checkout-index", "--all", "--force"]);
  return { root, sources };
}

for (const autocrlf of ["true", "false"]) {
  test(`dependency patch bytes survive a real Git checkout with autocrlf=${autocrlf}`, (t) => {
    const { root, sources } = checkout(t, autocrlf, attributes);
    assert.ok(sources.size > 0);
    for (const [name, source] of sources) {
      const checkedOut = readFileSync(path.join(root, "patches", name));
      assert.equal(hash(checkedOut), hash(source), `${name}: complete source hash`);
      assert.deepEqual(checkedOut, source, `${name}: complete source bytes`);
    }
  });
}

test("omitting the LF policy reproduces the Windows braces patch corruption", (t) => {
  const withoutPolicy = attributes.replace(/^patches\/\*\.patch text eol=lf\r?\n/mu, "");
  assert.notEqual(withoutPolicy, attributes, "the actual dependency patch policy must be present");
  const { root, sources } = checkout(t, "true", withoutPolicy);
  const original = sources.get("braces@3.0.3.patch");
  assert.ok(original);
  const checkedOut = readFileSync(path.join(root, "patches", "braces@3.0.3.patch"));
  assert.notEqual(hash(checkedOut), hash(original));
  assert.deepEqual(checkedOut, Buffer.from(original.toString("utf8").replaceAll("\n", "\r\n")));
  assert.match(checkedOut.toString("utf8"), /^diff --git [^\n]+\r\n/mu);
});
