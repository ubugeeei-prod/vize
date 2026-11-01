import assert from "node:assert/strict";
import test from "node:test";
import { precompileInputOwners, snapshotPrecompileInputOwners } from "./precompile-owner.ts";

void test("only active capabilities in the exact environment can own current input bytes", () => {
  const observed: unknown[] = [];
  const owner = {
    ownsInput(file: string, source: string) {
      assert.equal(this, owner);
      observed.push({ file, source });
      return source === "owned";
    },
  };
  const capability = {
    version: 1,
    forConfig: (config: object) => (config === client ? owner : undefined),
  };
  const plugin = {
    name: "arbitrary-companion-name",
    api: { vizePrecompileInputOwner: capability },
  };
  const client = { plugins: [plugin] };
  const server = { plugins: [{ name: "vite-plugin-musea" }] };
  const owners = precompileInputOwners(
    { environment: { getTopLevelConfig: () => client } },
    server,
  );
  assert.equal(owners.length, 1);
  assert.equal(owners[0]?.("/src/Raw.art.vue", "owned"), true);
  assert.equal(owners[0]?.("/src/Raw.art.vue", "changed"), false);
  assert.deepEqual(observed, [
    { file: "/src/Raw.art.vue", source: "owned" },
    { file: "/src/Raw.art.vue", source: "changed" },
  ]);
  assert.deepEqual(
    precompileInputOwners({ environment: { getTopLevelConfig: () => server } }, client),
    [],
  );
  assert.deepEqual(precompileInputOwners({}), []);
  assert.equal(precompileInputOwners({}, client).length, 1);
  assert.deepEqual(
    snapshotPrecompileInputOwners({ plugins: [plugin] }),
    [],
    "unregistered config objects cannot borrow ownership",
  );
});

void test("each run freezes the actual method instead of rereading a mutable API", () => {
  const owner = { ownsInput: () => true };
  const capability = { version: 1, forConfig: () => owner };
  const config = { plugins: [{ api: { vizePrecompileInputOwner: capability } }] };
  const first = snapshotPrecompileInputOwners(config);
  owner.ownsInput = () => false;
  assert.equal(first[0]?.("a", "b"), true);
  assert.equal(snapshotPrecompileInputOwners(config)[0]?.("a", "b"), false);
  assert.ok(Object.isFrozen(first));
});

void test("unsupported or malformed ownership never silently skips compilation", () => {
  for (const value of [null, true, { version: 2, forConfig() {} }, { version: 1 }]) {
    assert.throws(
      () =>
        snapshotPrecompileInputOwners({ plugins: [{ api: { vizePrecompileInputOwner: value } }] }),
      /input owner capability/,
    );
  }
  for (const value of [null, true, {}, { ownsInput: true }]) {
    assert.throws(
      () =>
        snapshotPrecompileInputOwners({
          plugins: [{ api: { vizePrecompileInputOwner: { version: 1, forConfig: () => value } } }],
        }),
      /Invalid resolved precompile input owner/,
    );
  }
  for (const result of [undefined, null, 1, "true", Promise.resolve(true)]) {
    const [owner] = snapshotPrecompileInputOwners({
      plugins: [
        {
          api: {
            vizePrecompileInputOwner: {
              version: 1,
              forConfig: () => ({ ownsInput: () => result }),
            },
          },
        },
      ],
    });
    assert.throws(() => owner?.("raw.art.vue", "source"), /must return a boolean/);
  }
  const failure = new Error("actual companion parse failure");
  const [owner] = snapshotPrecompileInputOwners({
    plugins: [
      {
        api: {
          vizePrecompileInputOwner: {
            version: 1,
            forConfig: () => ({
              ownsInput() {
                throw failure;
              },
            }),
          },
        },
      },
    ],
  });
  assert.throws(
    () => owner?.("raw.art.vue", "source"),
    (error) => error === failure,
  );
});
