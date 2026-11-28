import assert from "node:assert/strict";
import { test } from "node:test";
import { CompiledDependencyIndex } from "./compiled-dependency-index.ts";

const dependencies = (label: string, count = 32) =>
  Array.from({ length: count }, (_, index) => `/deps/${label}/${index}.ts`);

void test("starting shared ownership retains already observed direct snapshots and order", () => {
  const inputs = dependencies("transition");
  let observations = 0;
  const identities = new Set([inputs[0]]);
  const memberships = new Map([["old", identities]]);
  const index = new CompiledDependencyIndex(
    (file) => {
      observations++;
      return [file];
    },
    () => 0,
    { owners: new Map([[inputs[0], new Set(["old"])]]), memberships },
  );
  memberships.clear();
  assert.equal(observations, 0);
  index.add("first", inputs);
  index.add("second", inputs.slice());
  assert.equal(observations, inputs.length);
  assert.deepEqual([...index.ownersOf(inputs[0])], ["old", "first", "second"]);
  index.delete("old");
  index.add("old", [inputs[0]]);
  assert.deepEqual([...index.ownersOf(inputs[0])], ["first", "second", "old"]);
  assert.equal(observations, inputs.length + 1);
  index.clear();
  assert.deepEqual([...index.ownersOf(inputs[0])], []);
});

void test("identical large lists share observations and retain replacement order", () => {
  let observations = 0;
  const index = new CompiledDependencyIndex(
    (file) => {
      observations++;
      return [file];
    },
    () => 0,
  );
  const inputs = dependencies("shared");
  index.add("first", inputs);
  index.add("second", inputs.slice());
  assert.equal(observations, inputs.length);
  assert.deepEqual([...index.ownersOf(inputs[0])], ["first", "second"]);
  index.delete("first");
  index.add("first", inputs.slice());
  assert.deepEqual([...index.ownersOf(inputs[0])], ["second", "first"]);
  index.delete("second");
  assert.deepEqual([...index.ownersOf(inputs[0])], ["first"]);
  index.delete("first");
  assert.deepEqual([...index.ownersOf(inputs[0])], []);
});

void test("mutated input arrays and new filesystem epochs retain independent snapshots", () => {
  let generation = 0;
  const index = new CompiledDependencyIndex(
    (file) => [file, `${file}@${generation}`],
    () => generation,
  );
  const inputs = dependencies("mutable");
  const old = inputs[10];
  index.add("old-owner", inputs);
  inputs[10] = "/deps/replacement.ts";
  index.add("new-owner", inputs);
  assert.deepEqual([...index.ownersOf(old)], ["old-owner"]);
  assert.deepEqual([...index.ownersOf(inputs[10])], ["new-owner"]);
  generation++;
  index.add("next-epoch", inputs.slice());
  assert.deepEqual([...index.ownersOf(`${inputs[0]}@0`)], ["old-owner", "new-owner"]);
  assert.deepEqual([...index.ownersOf(`${inputs[0]}@1`)], ["next-epoch"]);
  index.clear();
  assert.deepEqual([...index.ownersOf(inputs[0])], []);
});

void test("standalone and cwd-relative lists always observe their identities again", () => {
  let observations = 0;
  const observe = (file: string) => {
    observations++;
    return [file];
  };
  const inputs = dependencies("standalone");
  const standalone = new CompiledDependencyIndex(observe);
  standalone.add("first", inputs);
  standalone.add("second", inputs);
  assert.equal(observations, inputs.length * 2);
  observations = 0;
  const relative = new CompiledDependencyIndex(observe, () => 0);
  const relativeInputs = inputs.map((file) => file.slice(1));
  relative.add("first", relativeInputs);
  relative.add("second", relativeInputs);
  assert.equal(observations, inputs.length * 2);
});

void test("collisions, direct entries and shared groups match a chronological ownership model", () => {
  let generation = 0;
  const index = new CompiledDependencyIndex(
    (file) => [file, `${file}@${generation}`],
    () => generation,
  );
  const model = new Map<string, { identities: Set<string>; order: number }>();
  let order = 0;
  const replace = (owner: string, inputs: string[]) => {
    index.delete(owner);
    index.add(owner, inputs);
    model.set(owner, {
      identities: new Set(inputs.flatMap((file) => [file, `${file}@${generation}`])),
      order: ++order,
    });
  };
  const inputs = Array.from({ length: 20 }, (_, group) => {
    const values = dependencies(`collision-${group}`);
    values[0] = "/deps/common-first.ts";
    values[values.length - 1] = "/deps/common-last.ts";
    return values;
  });
  const check = () => {
    for (const identity of [
      "/deps/common-first.ts",
      "/deps/common-last.ts",
      ...inputs.flat(),
      ...inputs.flat().map((file) => `${file}@${generation}`),
    ]) {
      const expected = [...model.entries()]
        .filter(([, value]) => value.identities.has(identity))
        .sort(([, left], [, right]) => left.order - right.order)
        .map(([owner]) => owner);
      assert.deepEqual([...index.ownersOf(identity)], expected, identity);
    }
  };
  for (let owner = 0; owner < 40; owner++) replace(`owner-${owner}`, inputs[owner % inputs.length]);
  replace("small-direct", ["/deps/common-first.ts"]);
  check();
  generation++;
  for (let owner = 39; owner >= 0; owner -= 3)
    replace(`owner-${owner}`, inputs[(owner + 1) % inputs.length]);
  for (let owner = 0; owner < 40; owner += 5) {
    index.delete(`owner-${owner}`);
    model.delete(`owner-${owner}`);
  }
  check();
});

void test("direct owners keep chronological order before, during and after shared groups", () => {
  const index = new CompiledDependencyIndex(
    (file) => [file],
    () => 0,
  );
  const shared = dependencies("epochs");
  const common = shared[0];
  const model = new Map<string, readonly string[]>();
  const replace = (owner: string, inputs: readonly string[]) => {
    index.delete(owner);
    index.add(owner, inputs);
    model.delete(owner);
    model.set(owner, inputs);
    assert.deepEqual(
      [...index.ownersOf(common)],
      [...model].filter(([, files]) => files.includes(common)).map(([file]) => file),
    );
  };
  const remove = (owner: string) => {
    index.delete(owner);
    model.delete(owner);
  };
  replace("small-before", [common]);
  replace("group-one", shared);
  replace("small-during", [common]);
  replace("group-two", shared);
  replace("small-before", [common]);
  remove("group-one");
  remove("group-two");
  replace("small-after", [common]);
  replace("next-group", shared);
  replace("small-during", [common]);
  replace("last-group", shared);
});

void test("many unique large lists retain exact ownership through fallback, replacement and clear", () => {
  const index = new CompiledDependencyIndex(
    (file) => [file],
    () => 0,
  );
  const common = "/deps/all.ts";
  const inputs = Array.from({ length: 270 }, (_, group) => {
    const values = dependencies(`unique-${group}`, 260);
    values[20] = common;
    return values;
  });
  const model = new Map<string, readonly string[]>();
  const replace = (owner: string, files: readonly string[]) => {
    index.delete(owner);
    index.add(owner, files);
    model.delete(owner);
    model.set(owner, files);
  };
  const check = () => {
    for (const identity of [common, inputs[0][10], inputs[255][10], inputs[269][10]]) {
      assert.deepEqual(
        [...index.ownersOf(identity)],
        [...model].filter(([, files]) => files.includes(identity)).map(([file]) => file),
      );
    }
  };
  for (let group = 0; group < inputs.length; group++) {
    replace(`first-${group}`, inputs[group]);
    replace(`second-${group}`, inputs[group].slice());
  }
  check();
  for (let group = 0; group < 260; group++) {
    for (const prefix of ["first", "second"]) {
      index.delete(`${prefix}-${group}`);
      model.delete(`${prefix}-${group}`);
    }
  }
  for (let group = 269; group >= 255; group--) replace(`third-${group}`, inputs[group]);
  check();
  index.clear();
  assert.deepEqual([...index.ownersOf(common)], []);
  replace("after-clear", inputs[0]);
  model.clear();
  model.set("after-clear", inputs[0]);
  check();
});

void test("a large shared owner set merges without exceeding the call argument limit", () => {
  const index = new CompiledDependencyIndex(
    (file) => [file],
    () => 0,
  );
  const inputs = dependencies("large-owner-set");
  const count = 150_000;
  for (let owner = 0; owner < count; owner++) index.add(`owner-${owner}`, inputs);
  index.add("direct-owner", [inputs[0]]);
  const owners = [...index.ownersOf(inputs[0])];
  assert.equal(owners.length, count + 1);
  assert.equal(owners[0], "owner-0");
  assert.equal(owners[count - 1], `owner-${count - 1}`);
  assert.equal(owners[count], "direct-owner");
});

void test("a unique pending list releases its direct edges before replacement or promotion", () => {
  const index = new CompiledDependencyIndex(
    (file) => [file],
    () => 0,
  );
  const inputs = dependencies("pending");
  index.add("removed", inputs);
  index.add("small", [inputs[0]]);
  index.delete("removed");
  assert.deepEqual([...index.ownersOf(inputs[0])], ["small"]);
  index.add("replacement", inputs);
  index.add("second", inputs.slice());
  assert.deepEqual([...index.ownersOf(inputs[0])], ["small", "replacement", "second"]);
  index.delete("replacement");
  index.delete("second");
  assert.deepEqual([...index.ownersOf(inputs[0])], ["small"]);
  index.add("replacement", inputs);
  index.delete("replacement");
  index.add("replacement", ["/deps/other.ts"]);
  assert.deepEqual([...index.ownersOf(inputs[0])], ["small"]);
});
