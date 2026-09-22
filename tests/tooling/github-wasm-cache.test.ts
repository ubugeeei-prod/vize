// The WASM build caches restore generated artifacts only. A cache step whose
// `path` covers a tracked file overwrites the checkout with the sources of
// whichever commit populated the cache: the playground's `src/wasm` directory
// holds both the wasm-bindgen outputs and tracked TS sources, and a
// whole-directory cache once restored a pre-#6290 `wasm-transform.ts` onto the
// #6290 merge, dropping the Spolvero feed on main.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = { name?: string; uses?: string; with?: Record<string, string> };
type Workflow = { jobs?: Record<string, { steps?: Step[] }> };

const CACHES = [
  { workflow: "check.yml", job: "playground-test" },
  { workflow: "build-docs.yml", job: "build-playground" },
] as const;

/** Every cached path is a directory's wasm-bindgen output glob. */
const OUTPUT_GLOB = /^(?:npm\/wasm|playground\/src\/wasm)\/vize_vitrine\*$/;

function trackedFiles(): string[] {
  const listed = spawnSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" });
  assert.equal(listed.status, 0, listed.stderr);
  return listed.stdout.split("\0").filter(Boolean);
}

function wasmCacheStep(workflow: string, job: string): Step {
  const parsed = parse(readRepoFile(".github", "workflows", workflow)) as Workflow;
  const step = parsed.jobs?.[job]?.steps?.find(
    (candidate) => candidate.name === "Cache WASM build",
  );
  assert.ok(step?.uses?.startsWith("actions/cache@"), `${workflow} ${job} caches its WASM build`);
  return step;
}

test("WASM caches never cover tracked files", () => {
  const tracked = trackedFiles();
  for (const { workflow, job } of CACHES) {
    const paths = (wasmCacheStep(workflow, job).with?.path ?? "")
      .split("\n")
      .map((line) => line.replace(/\s+#.*$/, "").trim())
      .filter(Boolean);
    assert.ok(paths.length > 0, `${workflow} ${job} names its cached paths`);
    for (const cached of paths) {
      assert.match(
        cached,
        OUTPUT_GLOB,
        `${workflow} ${job}: ${cached} is a wasm-bindgen output glob`,
      );
      const prefix = cached.slice(0, -1);
      const covered = tracked.filter((file) => file.startsWith(prefix));
      assert.deepEqual(
        covered,
        [],
        `${workflow} ${job}: cache path ${cached} covers tracked files`,
      );
    }
  }
});

test("WASM cache keys follow every input of the build", () => {
  for (const { workflow, job } of CACHES) {
    const key = wasmCacheStep(workflow, job).with?.key ?? "";
    for (const input of [
      "'crates/**/*.rs'",
      "'crates/**/Cargo.toml'",
      "'Cargo.toml'",
      "'Cargo.lock'",
      "'tools/moon/cmd/github/build_vitrine_wasm/**'",
    ]) {
      assert.ok(key.includes(input), `${workflow} ${job}: cache key hashes ${input}`);
    }
  }
});
