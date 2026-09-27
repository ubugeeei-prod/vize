import { execFileSync, spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const nativeDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const receiptPath = (directory) =>
  join(directory, ".artifacts", "native", "js-test-preparation.json");
const ownerPrefix = "--native-test-owner=";

function sourceIdentity(directory) {
  const git = (...args) =>
    execFileSync("git", args, { cwd: directory, maxBuffer: 64 * 1024 * 1024 });
  return {
    head: git("rev-parse", "HEAD").toString().trim(),
    tree: git("rev-parse", "HEAD^{tree}").toString().trim(),
    workingDiff: createHash("sha256")
      .update(git("diff", "--binary", "HEAD"))
      .digest("hex"),
  };
}

function bindingIdentity(directory) {
  const bindings = readdirSync(directory).filter(
    (name) => name.startsWith("vize-vitrine.") && name.endsWith(".node"),
  );
  if (bindings.length !== 1) throw new Error("Native preparation requires exactly one local addon");
  return {
    binding: bindings[0],
    sha256: createHash("sha256")
      .update(readFileSync(join(directory, bindings[0])))
      .digest("hex"),
  };
}

function ownerIsAlive(receipt) {
  try {
    if (!Number.isInteger(receipt.ownerPid) || receipt.ownerPid <= 0) return false;
    process.kill(receipt.ownerPid, 0);
    if (!new RegExp(`^${ownerPrefix}[a-f0-9-]{36}$`).test(receipt.ownerArgument)) return false;
    const ownerArgs = execFileSync("ps", ["-ww", "-p", String(receipt.ownerPid), "-o", "args="], {
      encoding: "utf8",
    })
      .trim()
      .split(/\s+/);
    return ownerArgs.includes(receipt.ownerArgument);
  } catch {
    return false;
  }
}

export function nativePreparationIsActive(directory) {
  try {
    const receipt = JSON.parse(readFileSync(receiptPath(directory), "utf8"));
    if (receipt.schemaVersion !== 1 || receipt.nativeDir !== realpathSync(directory)) return false;
    if (!ownerIsAlive(receipt)) return false;
    const source = sourceIdentity(directory);
    if (
      receipt.head !== source.head ||
      receipt.tree !== source.tree ||
      receipt.workingDiff !== source.workingDiff
    )
      return false;
    const identity = bindingIdentity(directory);
    return receipt.binding === identity.binding && receipt.sha256 === identity.sha256;
  } catch {
    return false;
  }
}

export function withPreparedNative(directory, ownerArgument, command, args, run = spawnSync) {
  const file = receiptPath(directory);
  const owner = { ownerPid: process.pid, ownerArgument };
  // The fixed root package command is a .cmd shim on Windows.
  const options = {
    cwd: process.cwd(),
    env: process.env,
    stdio: "inherit",
    shell: process.platform === "win32" && command === "vp",
  };
  if (!ownerIsAlive(owner)) {
    if (existsSync(file)) {
      try {
        const existing = JSON.parse(readFileSync(file, "utf8"));
        if (Number.isInteger(existing.ownerPid) && existing.ownerPid > 0) {
          process.kill(existing.ownerPid, 0);
          throw new Error("Cannot inspect an existing live native preparation owner");
        }
      } catch (error) {
        if (
          error instanceof Error &&
          error.message === "Cannot inspect an existing live native preparation owner"
        )
          throw error;
      }
    }
    console.log(
      "Native reuse is unavailable: running the original package tests with standalone Vite preparation.",
    );
    return run(command, args, options).status ?? 1;
  }
  const receipt = {
    schemaVersion: 1,
    ...owner,
    id: randomUUID(),
    nativeDir: realpathSync(directory),
    ...sourceIdentity(directory),
    ...bindingIdentity(directory),
  };
  if (existsSync(file)) {
    let existing;
    try {
      existing = JSON.parse(readFileSync(file, "utf8"));
    } catch {}
    if (existing && ownerIsAlive(existing)) throw new Error("Native preparation is already active");
    rmSync(file);
  }
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, `${JSON.stringify(receipt)}\n`, { flag: "wx" });
  try {
    return run(command, args, options).status ?? 1;
  } finally {
    if (existsSync(file) && JSON.parse(readFileSync(file, "utf8")).id === receipt.id) rmSync(file);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [first, ...rest] = process.argv.slice(2);
  if (!first) throw new Error("usage: test-preparation.mjs COMMAND [ARGS...]");
  if (first.startsWith(ownerPrefix)) {
    const [command, ...args] = rest;
    if (!command) throw new Error("Native preparation owner requires a command");
    process.exitCode = withPreparedNative(nativeDir, first, command, args);
  } else {
    process.exitCode =
      spawnSync(
        process.execPath,
        [fileURLToPath(import.meta.url), `${ownerPrefix}${randomUUID()}`, first, ...rest],
        { cwd: process.cwd(), env: process.env, stdio: "inherit" },
      ).status ?? 1;
  }
}
