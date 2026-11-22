// Compare the release-built browser package from two immutable revisions.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { smokeWasmPackage } from "../../support/compat/npm/smoke-wasm-package.ts";
import { compareWasmResponses } from "../../support/compat/npm/wasm-responses.ts";

const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();

interface Member {
  path: string;
  size: number;
  mode: number;
}

export function packBrowserPackage(directory: string, destination: string) {
  fs.mkdirSync(destination);
  const packed: { filename: string; name: string; version: string; files: Member[] }[] = JSON.parse(
    execFileSync(
      "npm",
      ["pack", "--offline", "--ignore-scripts", "--json", "--pack-destination", destination],
      { cwd: directory, encoding: "utf8" },
    ),
  );
  if (packed.length !== 1 || packed[0].name !== "@vizejs/wasm")
    throw new Error("expected one @vizejs/wasm archive");
  const [entry] = packed;
  const archive = path.join(destination, entry.filename);
  execFileSync("tar", ["-xzf", archive, "-C", destination]);
  const installed = path.join(destination, "package");
  const members = entry.files
    .map(({ path: name, size, mode }) => {
      const content = fs.readFileSync(path.join(installed, name));
      if (content.byteLength !== size) throw new Error("incorrect packed size for " + name);
      return { path: name, size, mode, sha256: digest(content) };
    })
    .sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  return {
    archive: entry.filename,
    version: entry.version,
    bytes: fs.statSync(archive).size,
    sha256: digest(fs.readFileSync(archive)),
    members,
  };
}

export function changedMembers(
  base: ReturnType<typeof packBrowserPackage>,
  head: ReturnType<typeof packBrowserPackage>,
): string[] {
  const before = new Map(base.members.map((m) => [m.path, m]));
  const after = new Map(head.members.map((m) => [m.path, m]));
  return [...new Set([...before.keys(), ...after.keys()])].sort().filter((name) => {
    const a = before.get(name);
    const b = after.get(name);
    return !a || !b || a.sha256 !== b.sha256 || a.mode !== b.mode || a.size !== b.size;
  });
}

async function main([base, head, report, ...extra]: string[]) {
  if (extra.length || !report || ![base, head].every((sha) => /^[a-f0-9]{40}$/u.test(sha))) {
    throw new Error("expected BASE_SHA HEAD_SHA REPORT.json");
  }
  const root = git(process.cwd(), "rev-parse", "--show-toplevel");
  for (const sha of [base, head]) {
    if (git(root, "cat-file", "-t", sha) !== "commit") throw new Error("expected commit " + sha);
  }
  const epoch = git(root, "show", "-s", "--format=%ct", base);
  const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-packed-browser-")));
  const source = path.join(temporary, "source");
  const baseDir = path.join(temporary, "base");
  const headDir = path.join(temporary, "head");
  const env = {
    ...process.env,
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: path.join(source, "target"),
    SOURCE_DATE_EPOCH: epoch,
  };
  // Each release builder receives its own committed sources and an empty target
  // at the same absolute path, including the build script and wasm-bindgen output.
  function build(destination: string) {
    const lock = digest(fs.readFileSync(path.join(source, "Cargo.lock")));
    execFileSync(process.execPath, ["tools/support/tasks/build-vize-wasm-package.ts"], {
      cwd: source,
      env,
      stdio: "inherit",
    });
    if (digest(fs.readFileSync(path.join(source, "Cargo.lock"))) !== lock)
      throw new Error("release builder changed Cargo.lock");
    return {
      revision: git(source, "rev-parse", "HEAD"),
      lock,
      ...packBrowserPackage(path.join(source, "npm/wasm"), destination),
    };
  }
  try {
    git(root, "worktree", "add", "--quiet", "--detach", source, base);
    let old;
    try {
      old = build(baseDir);
    } finally {
      git(root, "worktree", "remove", "--force", source);
    }
    git(root, "worktree", "add", "--quiet", "--detach", source, head);
    try {
      const current = build(headDir);
      const basePackage = path.join(baseDir, "package");
      const headPackage = path.join(headDir, "package");
      await smokeWasmPackage(basePackage);
      await smokeWasmPackage(headPackage);
      const responses = await compareWasmResponses(basePackage, headPackage, source);
      const changed = changedMembers(old, current);
      const sameArchive = old.sha256 === current.sha256;
      const evidence = {
        base: old,
        head: current,
        sourceDateEpoch: epoch,
        node: process.version,
        npm: execFileSync("npm", ["--version"], { encoding: "utf8" }).trim(),
        rustc: execFileSync("rustc", ["-vV"], { cwd: source, env, encoding: "utf8" }).trim(),
        bindgen: execFileSync("wasm-bindgen", ["--version"], { encoding: "utf8" }).trim(),
        sameArchive,
        changedMembers: changed,
        responses,
      };
      fs.writeFileSync(report, JSON.stringify(evidence, null, 2) + "\n");
      console.log(
        `@vizejs/wasm: ${current.members.length} packed files, ${current.bytes} archive bytes. Identical archive: ${sameArchive}. Changed members: ${JSON.stringify(changed)}. ${responses.fixtures} fixtures, ${responses.bytes} response bytes, ${responses.differences.length} differences.`,
      );
      return sameArchive && !changed.length && !responses.differences.length ? 0 : 1;
    } finally {
      git(root, "worktree", "remove", "--force", source);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  process.exitCode = await main(process.argv.slice(2));
