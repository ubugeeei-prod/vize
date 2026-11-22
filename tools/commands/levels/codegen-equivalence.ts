// Compare a library's linker inputs or the complete consuming executable for two immutable Git revisions.
// node tools/commands/levels/codegen-equivalence.ts BASE_SHA HEAD_SHA PACKAGE lib|bin:NAME default|portable REPORT.json
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Readable } from "node:stream";
import { isDeepStrictEqual } from "node:util";

interface Member {
  key: string;
  name: string;
  bytes: number;
  sha256: string;
}
interface Build {
  revision: string;
  manifest: string;
  target: { name: string; kind: string[]; source: string };
  lock: string;
  cargo: string;
  rustc: string;
  features: string[];
  profile: object;
  members: Member[];
}

interface CargoTarget {
  kind: string[];
  name: string;
  src_path: string;
}

interface CargoPackage {
  id: string;
  name: string;
  manifest_path: string;
  targets: CargoTarget[];
}

const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Compare Rust units by codegen-unit index and other bundled members by name. */
export function archiveKeys(names: string[]): { name: string; key: string }[] {
  const result = [];
  const keys = new Set<string>();
  let units = 0;
  for (const name of names) {
    // BSD ar lists its symbol index as a member. Its offsets describe the archive
    // layout, including the size of Rust metadata, rather than a linker input.
    if (name === "lib.rmeta" || /^__\.SYMDEF(?:_64)?(?: SORTED)?$/u.test(name)) continue;
    const unit = /-cgu\.(\d+)\.rcgu\.o$/u.exec(name);
    if (!unit && name.endsWith(".rcgu.o")) throw new Error(`unrecognized Rust object ${name}`);
    const key = unit ? `rust:${Number(unit[1])}` : `member:${name}`;
    if (keys.has(key)) throw new Error(`ambiguous archive member ${key}`);
    keys.add(key);
    if (unit) units++;
    result.push({ name, key });
  }
  if (!units) throw new Error("archive has no Rust codegen units");
  return result;
}

async function fingerprint(stream: Readable, key: string, name: string): Promise<Member> {
  const hash = createHash("sha256");
  let bytes = 0;
  for await (const chunk of stream) {
    bytes += chunk.length;
    hash.update(chunk);
  }
  if (!bytes) throw new Error(`empty product ${name}`);
  return { name, key, bytes, sha256: hash.digest("hex") };
}

async function readMembers(library: string): Promise<Member[]> {
  const names = execFileSync("ar", ["t", library], { encoding: "utf8" }).trimEnd().split("\n");
  const members = [];
  for (const { name, key } of archiveKeys(names)) {
    const child = spawn("ar", ["p", library, name], { stdio: ["ignore", "pipe", "inherit"] });
    const closed = once(child, "close");
    const member = await fingerprint(child.stdout, key, name);
    if ((await closed)[0] !== 0) throw new Error(`could not extract ${name}`);
    members.push(member);
  }
  return members;
}

async function build(
  checkout: string,
  target: string,
  pkg: string,
  product: string,
  mode: string,
  sourceDateEpoch: string,
): Promise<{ evidence: Build; executable?: string }> {
  const binary = product.startsWith("bin:") ? product.slice(4) : undefined;
  const metadata: { packages: CargoPackage[]; workspace_members: string[] } = JSON.parse(
    execFileSync("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"], {
      cwd: checkout,
      encoding: "utf8",
      maxBuffer: 128 * 1024 * 1024,
    }),
  );
  const packages = metadata.packages.filter(
    (entry) => entry.name === pkg && metadata.workspace_members.includes(entry.id),
  );
  if (packages.length !== 1) throw new Error(`expected one workspace package for ${pkg}`);
  const owner = packages[0];
  const targets = owner.targets.filter((target) =>
    binary
      ? target.name === binary && target.kind.includes("bin")
      : target.kind.includes("lib") || target.kind.includes("rlib"),
  );
  if (targets.length !== 1) throw new Error(`expected one ${product} target for ${pkg}`);
  const requested = targets[0];
  const selector = binary ? ["--bin", binary] : ["--lib"];
  const args = ["build", ...selector, "--locked", "--profile", "ci-opt", "-p", pkg];
  if (mode === "portable") args.push("--no-default-features");
  const output = execFileSync("cargo", [...args, "--message-format=json-render-diagnostics"], {
    cwd: checkout,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    maxBuffer: 128 * 1024 * 1024,
    env: {
      ...process.env,
      CARGO_TARGET_DIR: target,
      CARGO_INCREMENTAL: "0",
      SOURCE_DATE_EPOCH: sourceDateEpoch,
    },
  });
  const artifacts = output
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line))
    .filter(
      (m) =>
        m.reason === "compiler-artifact" &&
        m.package_id === owner.id &&
        m.manifest_path === owner.manifest_path &&
        m.target.name === requested.name &&
        m.target.src_path === requested.src_path &&
        isDeepStrictEqual(m.target.kind, requested.kind),
    );
  if (artifacts.length !== 1 || artifacts[0].fresh)
    throw new Error(`expected one freshly compiled workspace target for ${pkg}`);
  const artifact = artifacts[0];
  let members;
  if (binary) {
    if (!artifact.executable) throw new Error(`expected executable for ${binary}`);
    members = [await fingerprint(fs.createReadStream(artifact.executable), "executable", binary)];
  } else {
    const libraries = artifact.filenames.filter((name: string) => name.endsWith(".rlib"));
    if (libraries.length !== 1) throw new Error(`expected one rlib for ${pkg}`);
    members = await readMembers(libraries[0]);
  }
  const evidence = {
    revision: git(checkout, "rev-parse", "HEAD"),
    manifest: path.relative(checkout, artifact.manifest_path),
    target: {
      name: requested.name,
      kind: requested.kind,
      source: path.relative(checkout, requested.src_path),
    },
    cargo: execFileSync("cargo", ["-V"], { cwd: checkout, encoding: "utf8" }).trim(),
    rustc: execFileSync("rustc", ["-vV"], { cwd: checkout, encoding: "utf8" }).trim(),
    lock: digest(fs.readFileSync(path.join(checkout, "Cargo.lock"))),
    features: [...artifact.features].sort((left, right) => left.localeCompare(right, "en")),
    profile: artifact.profile,
    members,
  };
  return { evidence, executable: binary ? artifact.executable : undefined };
}

export function differences(base: Build, head: Build): string[] {
  const result = [];
  for (const field of [
    "manifest",
    "target",
    "lock",
    "cargo",
    "rustc",
    "features",
    "profile",
  ] as const)
    if (!isDeepStrictEqual(base[field], head[field])) result.push(field);
  if (
    !isDeepStrictEqual(
      base.members.map((m) => m.key),
      head.members.map((m) => m.key),
    )
  )
    result.push("archive membership or order");
  const old = new Map(base.members.map((m) => [m.key, m]));
  for (const member of head.members) {
    const prior = old.get(member.key);
    if (prior && (member.sha256 !== prior.sha256 || member.bytes !== prior.bytes))
      result.push(member.key);
  }
  return result;
}

async function main([base, head, pkg, product, mode, report, ...extra]: string[]) {
  if (
    extra.length ||
    !report ||
    ![base, head].every((rev) => /^[a-f0-9]{40}$/u.test(rev)) ||
    !/^[a-zA-Z0-9][a-zA-Z0-9_-]*$/u.test(pkg) ||
    !(product === "lib" || /^bin:[a-zA-Z0-9][a-zA-Z0-9_-]*$/u.test(product)) ||
    !["default", "portable"].includes(mode)
  )
    throw new Error("expected BASE_SHA HEAD_SHA PACKAGE lib|bin:NAME default|portable REPORT.json");
  const root = git(process.cwd(), "rev-parse", "--show-toplevel");
  for (const rev of [base, head])
    if (git(root, "cat-file", "-t", rev) !== "commit") throw new Error(`not a commit: ${rev}`);
  // Native dependencies embed __DATE__ and __TIME__. Both independent builds
  // must see the same clock as well as the same source and output paths.
  const sourceDateEpoch = git(root, "show", "-s", "--format=%ct", base);
  const temp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-codegen-")));
  const checkout = path.join(temp, "source");
  const target = path.join(temp, "target");
  async function compile(revision: string) {
    git(root, "worktree", "add", "--quiet", "--detach", checkout, revision);
    try {
      return await build(checkout, target, pkg, product, mode, sourceDateEpoch);
    } finally {
      git(root, "worktree", "remove", "--force", checkout);
    }
  }
  try {
    const first = await compile(base);
    const before = first.evidence;
    const saved = path.join(temp, "base-executable");
    if (first.executable) fs.copyFileSync(first.executable, saved, fs.constants.COPYFILE_FICLONE);
    // Both compilers see the same physical source and OUT_DIR paths. Each
    // starts from a fresh checkout and independently builds every dependency.
    fs.rmSync(target, { recursive: true });
    const second = await compile(head);
    const after = second.evidence;
    const changed = differences(before, after);
    let executables;
    if (second.executable && changed.includes("executable")) {
      const directory = fs.mkdtempSync(path.resolve(report) + ".executables-");
      try {
        for (const [side, source] of [
          ["base", saved],
          ["head", second.executable],
        ]) {
          fs.copyFileSync(source, path.join(directory, side), fs.constants.COPYFILE_FICLONE);
        }
      } catch (error) {
        fs.rmSync(directory, { recursive: true, force: true });
        throw error;
      }
      const relative = path.relative(path.dirname(path.resolve(report)), directory);
      executables = { base: path.join(relative, "base"), head: path.join(relative, "head") };
    }
    const evidence = {
      package: pkg,
      product,
      mode,
      sourceDateEpoch,
      base: before,
      head: after,
      differences: changed,
      executables,
    };
    fs.writeFileSync(report, JSON.stringify(evidence, null, 2) + "\n");
    const bytes = after.members.reduce((n, m) => n + m.bytes, 0);
    console.log(
      `${pkg} ${product} (${mode}): ${before.members.length} / ${after.members.length} members, ${bytes} head bytes, differences: ${JSON.stringify(changed)}`,
    );
    return changed.length ? 1 : 0;
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  process.exitCode = await main(process.argv.slice(2));
