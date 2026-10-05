import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  allocatorReplayText,
  contracts,
  hostModule,
  hostModuleText,
  originalAllocatorSelection,
  prepareAllocatorSelection,
  profileExportContract,
  sfcImportGate,
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

test("original allocator selection replays to exact historical allocator bytes", () => {
  const planned = prepareAllocatorSelection(reader(original));
  assert.equal(planned.size, 7);
  for (const [file, text] of planned)
    assert.equal(text, allocatorReplayText(file, current.get(file)));
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
    for (const { file } of contracts.slice(2, 4)) {
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

test("allocator replay preflights the real SFC source gate before writes", () => {
  for (const state of [original, current]) {
    for (const altered of [
      state
        .get(sfcImportGate)!
        .replace("manifest.dependencies.vize_l0", "manifest.dependencies.vize_carton"),
      state.get(sfcImportGate)!.replace("assert.doesNotMatch", "assert.match"),
      state.get(sfcImportGate)! + "\n// stale source policy\n",
    ]) {
      const changed = new Map(state);
      changed.set(sfcImportGate, altered);
      rejectsBeforeWrites(changed);
    }
  }
});

test("allocator replay preflights the existing exporter exact-gate contract", () => {
  for (const state of [original, current]) {
    const changed = new Map(state);
    changed.set(
      profileExportContract,
      state
        .get(profileExportContract)!
        .replace(
          "0039b68bc23fa3077a61407c0caf903a978e81027377c99563142120008829f1",
          "unexpected_gate_digest",
        ),
    );
    rejectsBeforeWrites(changed);
  }
});

void test("allocator checks preserve later exact exporters while old replay retains its own bytes", () => {
  const source = current.get(profileExportContract)!;
  const historical = allocatorReplayText(profileExportContract, source)!;
  assert.notEqual(historical, source);
  assert.equal(prepareAllocatorSelection(reader(current)).size, 0);
  const retained = new Map(current);
  retained.set(profileExportContract, historical);
  assert.equal(prepareAllocatorSelection(reader(retained)).size, 0);
  assert.equal(current.get(profileExportContract), source);
  for (const changed of [
    source + "\n// partial snapshot contract\n",
    source.replace(
      "3d1747c3dcfcf4ec8bc80ec722a6bd58d488cfdef2c7de84374a526417ebc9f9",
      "unknown_owned_exporter",
    ),
  ]) {
    const altered = new Map(current);
    altered.set(profileExportContract, changed);
    rejectsBeforeWrites(altered);
  }
});
