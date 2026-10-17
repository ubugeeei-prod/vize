import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

/**
 * Recreates an Oxlint fixture project as an isolated git repository root.
 *
 * These fixtures live under the workspace's `target/` directory, which the
 * repository `.gitignore` excludes. Oxlint resolves the enclosing git
 * repository at walk time and honours its ignore rules, so without a boundary
 * every fixture file is filtered out and the run reports "No files found to
 * lint" instead of the diagnostics under test. `--no-ignore` does not help: it
 * only disables `.eslintignore` sources.
 *
 * An empty `.git` directory is enough to make the walker treat the fixture as
 * its own repository, which is also what a real consumer project looks like.
 */
export function resetFixtureDir(fixtureDir: string): void {
  fs.rmSync(fixtureDir, { force: true, recursive: true });
  fs.mkdirSync(path.join(fixtureDir, ".git"), { recursive: true });
}

/** Keep unchanged host argv outside the repository's inherited ignore rules. */
export function createExternalFixtureDir(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-original-host-"));
  const relative = path.relative(path.resolve(import.meta.dirname, "../../../.."), root);
  if (path.isAbsolute(relative) || relative.split(path.sep)[0] === "..") return root;
  fs.rmSync(root, { recursive: true, force: true });
  throw new Error("The host fixture temp directory must be outside the source repository.");
}

/** Archive only declared physical inputs; never follow dependency symlinks. */
export function captureFixtureInputs(root: string, files: string[], output: string) {
  fs.mkdirSync(output, { recursive: true });
  return files.map((filename) => {
    assert.equal(path.basename(filename), filename);
    const file = path.join(root, filename);
    assert.ok(fs.lstatSync(file).isFile(), `fixture input must be a regular file: ${file}`);
    const bytes = fs.readFileSync(file);
    fs.writeFileSync(path.join(output, filename), bytes, { flag: "wx" });
    return { filename, bytes: bytes.length, sha256: sha256(bytes) };
  });
}

/** The fixed public baseline must resolve entirely inside its own installed project. */
export function authenticatePublicNative(
  options: { pluginPackage: string; oxlintPackage: string; output: string },
  imagePath: string,
) {
  const root = fs.realpathSync(path.resolve(options.pluginPackage, "../.."));
  for (const [directory, name] of [
    [options.pluginPackage, "oxlint-plugin-vize"],
    [options.oxlintPackage, "oxlint"],
  ])
    assert.equal(
      fs.realpathSync(directory!),
      path.join(root, "node_modules", name!),
      "public host packages must belong to the same actual installation",
    );
  const relative = path.relative(root, fs.realpathSync(imagePath));
  assert.ok(
    !path.isAbsolute(relative) && relative.split(path.sep)[0] !== "..",
    "public native image cannot resolve outside its installation",
  );
  let directory = path.dirname(imagePath);
  while (!fs.existsSync(path.join(directory, "package.json"))) {
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory);
    assert.notEqual(directory, root);
    directory = parent;
  }
  const bytes = fs.readFileSync(path.join(directory, "package.json"));
  const manifest = JSON.parse(bytes.toString("utf8"));
  assert.ok(manifest.name.startsWith("@vizejs/native-"));
  assert.equal(manifest.version, "0.458.0");
  const lockBytes = fs.readFileSync(path.join(root, "package-lock.json"));
  const key = path.relative(root, directory).split(path.sep).join("/");
  const locked = JSON.parse(lockBytes.toString("utf8")).packages[key];
  assert.equal(locked.version, "0.458.0");
  assert.match(locked.integrity, /^sha512-/u);
  assert.ok(locked.resolved.startsWith("https://registry.npmjs.org/"));
  fs.writeFileSync(path.join(options.output, "public-native-package.json"), bytes, { flag: "wx" });
  fs.writeFileSync(path.join(options.output, "public-package-lock.json"), lockBytes, {
    flag: "wx",
  });
  return {
    root,
    package: manifest.name,
    version: manifest.version,
    manifestSha256: sha256(bytes),
    lockKey: key,
    lockSha256: sha256(lockBytes),
    integrity: locked.integrity,
    resolved: locked.resolved,
  };
}
