import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
  rmdirSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

function git(root: string, args: string[]) {
  return execFileSync("git", ["-c", "core.hooksPath=/dev/null", "-C", root, ...args], {
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  }).trim();
}

function privatePath(root: string, path: string) {
  // Reject a redirected fixture or ancestor before creating or removing anything.
  let current = path;
  while (current !== root) {
    try {
      if (lstatSync(current).isSymbolicLink()) throw new Error("Fixture path crosses a symlink");
    } catch (error: unknown) {
      if (!(error instanceof Error) || !("code" in error) || error.code !== "ENOENT") throw error;
    }
    const parent = dirname(current);
    if (parent === current) throw new Error("Fixture must be inside its source checkout");
    current = parent;
  }
}

function sourceInput(root: string, fixture: string) {
  if (
    [
      "GIT_DIR",
      "GIT_WORK_TREE",
      "GIT_COMMON_DIR",
      "GIT_OBJECT_DIRECTORY",
      "GIT_ALTERNATE_OBJECT_DIRECTORIES",
      "GIT_INDEX_FILE",
      "GIT_SHALLOW_FILE",
    ].some((name) => process.env[name] !== undefined)
  )
    throw new Error("Fixture hydration requires a private Git context");
  if (!/^tests\/_fixtures\/_git\/[A-Za-z0-9_-]+$/.test(fixture))
    throw new Error("Expected a single source-tree fixture gitlink");
  if (realpathSync(root) !== root) throw new Error("Source checkout must be canonical");
  privatePath(root, join(root, fixture));
  const row = git(root, ["ls-tree", "HEAD", "--", fixture]);
  const head = /^160000 commit ([a-f0-9]{40})\t/.exec(row)?.[1];
  if (!head) throw new Error("Fixture must be a committed gitlink");
  if (git(root, ["diff", "HEAD", "--", ".gitmodules"]))
    throw new Error("Fixture declarations differ from source HEAD");
  const declaration = git(root, [
    "config",
    "--blob",
    "HEAD:.gitmodules",
    "--get",
    `submodule.${fixture}.url`,
  ]);
  if (!declaration || /[\r\n\0]/.test(declaration)) throw new Error("Fixture origin is invalid");
  // Let the original Git protocol resolve relative origins and existing local
  // overrides; never reimplement .gitmodules URL resolution in the new tool.
  git(root, ["submodule", "init", "--", fixture]);
  const url = git(root, ["config", "--get", `submodule.${fixture}.url`]);
  return {
    checkout: git(root, ["rev-parse", "HEAD"]),
    head,
    url,
    declaration,
    modules: git(root, ["rev-parse", "HEAD:.gitmodules"]),
  };
}
type SourceInput = ReturnType<typeof sourceInput>;

function unchanged(root: string, fixture: string, input: SourceInput) {
  if (JSON.stringify(sourceInput(root, fixture)) !== JSON.stringify(input))
    throw new Error("Source fixture identity changed during hydration");
}

function validate(root: string, input: SourceInput) {
  if (git(root, ["rev-parse", "HEAD"]) !== input.head)
    throw new Error("Fixture HEAD differs from the source gitlink");
  // The stored URL is authoritative; the runner's existing transport rewrite
  // may legitimately map it to a local Git proxy during fetch.
  if (git(root, ["config", "--get", "remote.origin.url"]) !== input.url)
    throw new Error("Fixture origin differs from the source declaration");
  if (git(root, ["status", "--porcelain=v1", "--untracked-files=all"]))
    throw new Error("Fixture checkout is dirty");
  const database = git(root, ["rev-parse", "--absolute-git-dir"]);
  if (existsSync(join(database, "objects/info/alternates")))
    throw new Error("Fixture must not borrow another object database");
  git(root, ["fsck", "--strict", "--no-reflogs", "--no-dangling", input.head]);
  return git(root, ["rev-parse", "HEAD^{tree}"]);
}

function canonicalFallback(root: string, fixture: string) {
  // Keep the original transport available; no failed fetch authorizes a fixture.
  execFileSync(
    "git",
    ["-C", root, "submodule", "update", "--init", "--depth", "1", "--", fixture],
    {
      stdio: "inherit",
    },
  );
}

function exactFetchUnavailable(stderr: string, head: string) {
  // Only Git's specific unsupported exact-object responses permit the original
  // transport. Unknown failures (including corrupt packs) remain failures.
  return [
    `Server does not allow request for unadvertised object ${head}`,
    `remote error: upload-pack: not our ref ${head}`,
    `couldn't find remote ref ${head}`,
  ].some((message) => stderr.trim() === `fatal: ${message}`);
}

export function hydratePinnedFixture(sourceRoot: string, fixture: string) {
  const started = performance.now();
  const root = resolve(sourceRoot);
  const input = sourceInput(root, fixture);
  const destination = join(root, fixture);
  const modulePath = resolve(root, git(root, ["rev-parse", "--git-path", `modules/${fixture}`]));
  let fetchMs: number | null = null;
  let verificationMs = 0;
  const verify = (path: string) => {
    const verificationStarted = performance.now();
    const tree = validate(path, input);
    verificationMs += performance.now() - verificationStarted;
    return tree;
  };
  let mode: "direct" | "fallback" | "existing" = "direct";
  if (existsSync(destination) && readdirSync(destination).length) {
    // Never reset a pre-existing or dirty fixture to manufacture a successful hit.
    verify(destination);
    mode = "existing";
  } else if (isAbsolute(input.url) || !input.url.includes(":") || existsSync(modulePath)) {
    // Git ignores clone --depth for plain local paths. Preserve that original
    // history protocol; existing metadata also belongs to Git's reuse path.
    canonicalFallback(root, fixture);
    mode = "fallback";
  } else {
    mkdirSync(dirname(destination), { recursive: true });
    const temporary = mkdtempSync(`${destination}.prepare-`);
    try {
      git(root, ["init", "--template=", temporary]);
      git(temporary, ["remote", "add", "origin", input.url]);
      const fetchStarted = performance.now();
      const fetched = spawnSync(
        "git",
        [
          "-c",
          "core.hooksPath=/dev/null",
          "-C",
          temporary,
          "fetch",
          "--depth",
          "1",
          "--no-tags",
          "origin",
          input.head,
        ],
        {
          stdio: ["ignore", "pipe", "pipe"],
          maxBuffer: 16 * 1024 * 1024,
          env: { ...process.env, LC_ALL: "C" },
        },
      );
      fetchMs = performance.now() - fetchStarted;
      if (fetched.stdout) process.stdout.write(fetched.stdout);
      if (fetched.stderr) process.stderr.write(fetched.stderr);
      if (fetched.error || fetched.signal)
        throw new Error("Exact fixture fetch could not complete", { cause: fetched.error });
      if (fetched.status !== 0) {
        if (
          (fetched.stdout?.length ?? 0) !== 0 ||
          !exactFetchUnavailable(fetched.stderr?.toString("utf8") ?? "", input.head)
        )
          throw new Error(
            "Exact fixture fetch failed; original hydration cannot waive this failure",
          );
        console.error(
          "Exact-pin fetch unavailable; using the original submodule hydration command.",
        );
        unchanged(root, fixture, input);
        canonicalFallback(root, fixture);
        mode = "fallback";
      } else {
        if (git(temporary, ["rev-parse", "FETCH_HEAD"]) !== input.head)
          throw new Error("Exact fetch returned a different fixture commit");
        git(temporary, ["checkout", "--detach", input.head]);
        verify(temporary);
        if (
          git(temporary, ["rev-parse", "--is-shallow-repository"]) !== "true" ||
          git(temporary, ["rev-list", "--count", "HEAD"]) !== "1"
        )
          throw new Error("Exact fixture fetch did not preserve depth one");
        unchanged(root, fixture, input);
        privatePath(root, destination);
        if (existsSync(destination)) {
          if (readdirSync(destination).length)
            throw new Error("Fixture destination changed during fetch");
          // This is the empty checkout-created gitlink directory, never source files.
          rmdirSync(destination);
        }
        renameSync(temporary, destination);
        git(root, ["submodule", "init", "--", fixture]);
        git(root, ["submodule", "absorbgitdirs", "--", fixture]);
      }
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
  }
  const tree = verify(destination);
  unchanged(root, fixture, input);
  const database = git(destination, ["rev-parse", "--absolute-git-dir"]);
  if (
    !lstatSync(join(destination, ".git")).isFile() ||
    realpathSync(database) !== realpathSync(modulePath)
  )
    throw new Error("Fixture did not retain its normal private submodule topology");
  const result = {
    schema: "vize-direct-pinned-fixture-v1",
    head: input.head,
    tree,
    mode,
    fetchMs,
    verificationMs,
    elapsedMs: performance.now() - started,
  };
  console.log(JSON.stringify(result));
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [root, fixture, ...extra] = process.argv.slice(2);
  try {
    if (!root || !fixture || extra.length)
      throw new Error("Usage: hydrate-pinned-fixture.ts <source-checkout> <fixture-gitlink>");
    hydratePinnedFixture(root, fixture);
  } catch (error: unknown) {
    console.error(error);
    process.exitCode = 1;
  }
}
