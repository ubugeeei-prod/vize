import assert from "node:assert/strict";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import type { InputState } from "./after-plan.ts";
import { bytes, sha256, sourcePath } from "./value.ts";
import type { Sink } from "./evidence.ts";
import type { ManagerFile } from "./optional-residency.ts";
export function checkCaseManagers(
  initial: ManagerFile[],
  actual: ManagerFile[],
  expectedCaseManifest?: Buffer,
): void {
  assert.deepEqual(
    actual.map((f) => f.path),
    initial.map((f) => f.path),
  );
  assert.equal(initial[0]?.path, "package.json");
  if (expectedCaseManifest === undefined) {
    assert.deepEqual(actual, initial);
    return;
  }
  assert.deepEqual(
    actual.slice(1),
    initial.slice(1),
    "lock/workspace/npm settings must retain initial bytes",
  );
  assert.deepEqual(actual[0], {
    path: "package.json",
    state: "PRESENT",
    rawBase64: expectedCaseManifest.toString("base64"),
    sha256: sha256(expectedCaseManifest),
  });
}
function exists(path: string) {
  try {
    return lstatSync(path);
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT")
      return null;
    throw error;
  }
}
function sourceFiles(root: string, universe?: string[]): string[] {
  const found: string[] = [];
  function walk(relative: string): void {
    const target = join(root, relative),
      stat = exists(target);
    if (!stat) return;
    assert(!stat.isSymbolicLink(), `source symlink refused: ${relative}`);
    if (stat.isFile()) {
      found.push(relative);
      return;
    }
    assert(stat.isDirectory());
    assert(
      relative === "src" ||
        universe === undefined ||
        universe.some((p) => p.startsWith(relative + "/")),
      `unexpected source directory retained/refused: ${relative}`,
    );
    for (const entry of readdirSync(target)) walk(relative + "/" + entry);
  }
  walk("src");
  return found.sort();
}
export function inputStateWitness(root: string, state: InputState): unknown {
  return {
    root,
    expectedStateSha256: sha256(JSON.stringify(state)),
    universe: state.universe,
    sources: sourceFiles(root),
    files: state.universe.map((path) => {
      const stat = exists(join(root, path));
      if (!stat) return { path, state: "ABSENT" };
      if (!stat.isFile() || stat.isSymbolicLink()) return { path, state: "UNEXPECTED_KIND" };
      assert(stat.size <= 1024 * 1024);
      const raw = readFileSync(join(root, path));
      return {
        path,
        state: "PRESENT",
        bytes: raw.length,
        sha256: sha256(raw),
        rawBase64: raw.toString("base64"),
      };
    }),
  };
}
export function assertInputState(root: string, state: InputState): void {
  assert.deepEqual(
    sourceFiles(root, state.universe),
    state.present
      .filter((f) => f.path.startsWith("src/"))
      .map((f) => f.path)
      .sort(),
  );
  assert.equal(new Set(state.universe).size, state.universe.length);
  for (const path of state.universe) {
    sourcePath(path);
    const stat = exists(join(root, path)),
      expected = state.present.find((f) => f.path === path);
    if (expected) {
      assert(stat?.isFile() && !stat.isSymbolicLink(), `expected regular owned input: ${path}`);
      assert(
        readFileSync(join(root, path)).equals(bytes(expected.base64)),
        `complete input differs: ${path}`,
      );
    } else {
      assert(state.absent.includes(path));
      assert.equal(stat, null, `owned input must be absent: ${path}`);
    }
  }
}
/** Restore only registered source inputs. Unknown bytes and unowned files survive
 * as evidence and refuse before any overwrite or deletion is attempted. */
export function restoreInputState(
  root: string,
  state: InputState,
  registered: InputState[],
  sink: Sink,
  beforeMutation?: (registeredCurrent: InputState) => void,
): void {
  sink("input-state-before-transition", inputStateWitness(root, state));
  assert(registered.length > 0);
  assert(registered.every((s) => JSON.stringify(s.universe) === JSON.stringify(state.universe)));
  assert(
    sourceFiles(root, state.universe).every((path) => state.universe.includes(path)),
    "unexpected source input retained/refused",
  );
  const current = registered.find((candidate) => {
    try {
      assertInputState(root, candidate);
      return true;
    } catch {
      return false;
    }
  });
  assert(current, "unexpected complete input state retained/refused");
  beforeMutation?.(current);
  for (const path of state.universe) {
    sourcePath(path);
    assert(
      path === "package.json" ||
        path === "tsconfig.json" ||
        /^src\/[\w./-]+\.(?:vue|ts)$/u.test(path),
    );
    const stat = exists(join(root, path));
    if (!stat) continue;
    assert(stat.isFile() && !stat.isSymbolicLink(), `unexpected owned path kind: ${path}`);
    const raw = readFileSync(join(root, path));
    assert(
      registered.some((s) => s.present.some((f) => f.path === path && raw.equals(bytes(f.base64)))),
      `unexpected owned bytes retained/refused: ${path}`,
    );
  }
  for (const path of state.absent) {
    assert(path.startsWith("src/"), "root manager/configuration inputs cannot be removed");
    if (exists(join(root, path))) unlinkSync(join(root, path));
  }
  for (const file of state.present) {
    const target = join(root, file.path),
      raw = bytes(file.base64);
    mkdirSync(dirname(target), { recursive: true });
    if (!exists(target) || !readFileSync(target).equals(raw)) writeFileSync(target, raw);
  }
  sink("input-state-after-transition", inputStateWitness(root, state));
  assertInputState(root, state);
}
