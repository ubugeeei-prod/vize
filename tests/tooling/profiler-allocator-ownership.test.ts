import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  contracts,
  hostModule,
  hostModuleText,
  originalAllocatorSelection,
  prepareAllocatorSelection,
} from "../../tools/support/levels/select-profile-allocator-host.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const current = new Map(
  [...contracts.map(({ file }) => file), hostModule].map((file) => [
    file,
    fs.readFileSync(path.join(root, file), "utf8"),
  ]),
);
const reader = (files: Map<string, string>) => (file: string) => files.get(file);
const original = originalAllocatorSelection(reader(current));

test("host allocator selection checks and repeats without writing", () => {
  assert.equal(prepareAllocatorSelection(reader(current)).size, 0);
  assert.equal(prepareAllocatorSelection(reader(current)).size, 0);
});

test("original allocator selection replays to exact current bytes", () => {
  const planned = prepareAllocatorSelection(reader(original));
  assert.equal(planned.size, 5);
  for (const [file, text] of planned) assert.equal(text, current.get(file));
  const replayed = new Map([...original, ...planned]);
  assert.equal(prepareAllocatorSelection(reader(replayed)).size, 0);
});

function rejectsBeforeWrites(files: Map<string, string>) {
  const writes: string[] = [];
  assert.throws(() => {
    for (const [file] of prepareAllocatorSelection(reader(files))) writes.push(file);
  }, /changed, missing, colliding or partial/);
  assert.deepEqual(writes, []);
}

test("missing, partial and colliding allocator states reject before writes", () => {
  for (const state of [original, current]) {
    for (const { file } of contracts) {
      const missing = new Map(state);
      missing.delete(file);
      rejectsBeforeWrites(missing);
      const partial = new Map(state);
      partial.set(file, state === original ? current.get(file)! : original.get(file)!);
      rejectsBeforeWrites(partial);
    }
  }
  const collision = new Map(original);
  collision.set(hostModule, hostModuleText);
  rejectsBeforeWrites(collision);
  const missingHost = new Map(current);
  missingHost.delete(hostModule);
  rejectsBeforeWrites(missingHost);
});

test("changed allocator arguments and host selection reject before writes", () => {
  for (const state of [original, current]) {
    const alteredHook = new Map(state);
    const file = contracts[0].file;
    alteredHook.set(
      file,
      state.get(file)!.replace("self.inner.alloc(layout)", "self.inner.alloc_zeroed(layout)"),
    );
    rejectsBeforeWrites(alteredHook);
    for (const { file } of contracts.slice(2)) {
      const alteredCaller = new Map(state);
      alteredCaller.set(
        file,
        state
          .get(file)!
          .replace("ProfilingAllocator<System>", "ProfilingAllocator<WrongAllocator>"),
      );
      rejectsBeforeWrites(alteredCaller);
    }
  }
  const alteredHost = new Map(current);
  alteredHost.set(
    hostModule,
    hostModuleText.replace("from_allocator(System)", "from_allocator(WrongAllocator)"),
  );
  rejectsBeforeWrites(alteredHost);
});

test("removing host convenience preserves every existing allocator hook", () => {
  const file = contracts[0].file;
  const before = original.get(file)!;
  const after = current.get(file)!;
  const hooks = "// SAFETY: Every method delegates to the wrapped allocator";
  assert.equal(before.slice(before.indexOf(hooks)), after.slice(after.indexOf(hooks)));
  assert.match(after, /use std::alloc::\{GlobalAlloc, Layout\};/u);
  assert.doesNotMatch(after, /\bSystem\b|pub const fn new\(|impl Default for ProfilingAllocator/u);
  assert.match(hostModuleText, /pub const fn system_allocator\(\) -> ProfilingAllocator<System>/u);
});
