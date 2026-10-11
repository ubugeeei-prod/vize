import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export interface FixtureBlob {
  path: string;
  mode: string;
  object: string;
}

export interface PinnedFixtureSource {
  revision: string;
  blobs: FixtureBlob[];
  gitlinks: { path: string; revision: string }[];
  inventorySha256: string;
}

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" });
}

export function sha256(bytes: string | Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** An empty submodule directory must never authenticate its parent repository. */
export function readPinnedFixtureSource(
  sourceDir: string,
  expectedRevision: string,
): PinnedFixtureSource {
  assert.match(expectedRevision, /^[a-f0-9]{40}$/, "fixture must have a complete Git pin");
  assert.equal(
    fs.realpathSync(git(sourceDir, ["rev-parse", "--show-toplevel"]).trim()),
    fs.realpathSync(sourceDir),
    "fixture must be an initialized repository, not its parent checkout",
  );
  const revision = git(sourceDir, ["rev-parse", "HEAD"]).trim();
  assert.equal(revision, expectedRevision, "fixture checkout must match the owning Gitlink");
  const gitlinks: PinnedFixtureSource["gitlinks"] = [];
  const blobs = git(sourceDir, ["ls-tree", "-r", "-z", revision])
    .split("\0")
    .filter(Boolean)
    .flatMap((record): FixtureBlob[] => {
      const separator = record.indexOf("\t");
      const [mode, kind, object] = record.slice(0, separator).split(" ");
      const relativePath = record.slice(separator + 1);
      assert.ok(
        !path.isAbsolute(relativePath) && !relativePath.split("/").includes(".."),
        `fixture path must remain inside the source root: ${relativePath}`,
      );
      if (kind === "commit") {
        assert.equal(mode, "160000");
        const nested = readPinnedFixtureSource(path.join(sourceDir, relativePath), object);
        gitlinks.push({ path: relativePath, revision: nested.revision });
        gitlinks.push(
          ...nested.gitlinks.map((entry) => ({ ...entry, path: `${relativePath}/${entry.path}` })),
        );
        return nested.blobs.map((blob) => ({ ...blob, path: `${relativePath}/${blob.path}` }));
      }
      assert.equal(kind, "blob");
      assert.ok(["100644", "100755", "120000"].includes(mode), `file mode in ${relativePath}`);
      return [{ path: relativePath, mode, object }];
    });
  assert.ok(blobs.length > 0, "fixture source tree must contain files");
  return {
    revision,
    blobs,
    gitlinks,
    inventorySha256: sha256(JSON.stringify({ blobs, gitlinks })),
  };
}

/** git archive omits nested repositories; export each authenticated Gitlink too. */
export function exportPinnedFixtureGitlinks(
  root: string,
  sourceDir: string,
  source: PinnedFixtureSource,
): void {
  for (const gitlink of source.gitlinks) {
    const target = path.join(root, gitlink.path);
    fs.mkdirSync(target, { recursive: true });
    const temporary = fs.mkdtempSync(path.join(path.dirname(root), ".vize-gitlink-"));
    try {
      const archive = path.join(temporary, "source.tar");
      execFileSync("git", ["archive", "--format=tar", `--output=${archive}`, gitlink.revision], {
        cwd: path.join(sourceDir, gitlink.path),
      });
      execFileSync("tar", ["-xf", archive], { cwd: target });
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  }
}

/** Authenticate every original file, including manifests, locks and config imports. */
export function assertPinnedFixtureFiles(cwd: string, source: PinnedFixtureSource): void {
  for (const blob of source.blobs) {
    const filePath = path.join(cwd, blob.path);
    const stat = fs.lstatSync(filePath);
    let bytes: Buffer;
    if (blob.mode === "120000") {
      assert.ok(stat.isSymbolicLink(), `original symlink changed: ${blob.path}`);
      bytes = Buffer.from(fs.readlinkSync(filePath));
    } else {
      assert.ok(stat.isFile(), `original source file changed type: ${blob.path}`);
      if (process.platform !== "win32") {
        assert.equal(
          Boolean(stat.mode & 0o111),
          blob.mode === "100755",
          `original executable mode changed: ${blob.path}`,
        );
      }
      bytes = fs.readFileSync(filePath);
    }
    const object = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
    assert.equal(object, blob.object, `original fixture bytes changed: ${blob.path}`);
  }
}
