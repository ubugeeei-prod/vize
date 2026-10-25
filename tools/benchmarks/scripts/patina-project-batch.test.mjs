import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertProcess,
  assertProfile,
  cleanEnvironment,
  inventory,
  makeCorpus,
  median,
  parseRss,
  profileCounts,
  readAuthority,
  sha256,
} from "./patina-project-batch-lib.mjs";

const fixture = fileURLToPath(new URL("../fixtures/patina-project-batch/", import.meta.url));
const frozen = readAuthority(fixture);

function temporary(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "patina-batch-contract-"));
  try {
    return run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("three captured templates reproduce all121 source files and exact1210-file public output", () => {
  assert.equal(frozen.seed.size, 121);
  assert.equal(frozen.rows.length, 1210);
  assert.equal(
    frozen.rows.reduce((sum, row) => sum + row.messages.length, 0),
    2820,
  );
  assert.equal(frozen.stdout.length, 1704822);
  assert.equal(
    sha256(frozen.stdout),
    "bf3a6089ca4026480ebcf5b188ae11f1e757c6067f59eea0fcb3ca121bcc50dc",
  );
  assert.equal(frozen.stdout.at(-1), "]".charCodeAt(0));
  assert.deepEqual(
    [...frozen.seed].filter(([name]) => name.endsWith("use-counter.ts")).map(([name]) => name),
    ["src/use-counter.ts"],
  );
});

for (const name of ["component.vue.txt", "module.ts.txt", "cohort.stdout", "package-lock.json"]) {
  test(`changed frozen ${name} fails before source generation`, () =>
    temporary((root) => {
      fs.cpSync(fixture, root, { recursive: true });
      fs.appendFileSync(path.join(root, name), " ");
      assert.throws(
        () => readAuthority(root),
        (error) => {
          assert.equal(error.code, "ERR_ASSERTION");
          assert.equal(error.message.split("\n")[0], `frozen artifact changed: ${name}`);
          return true;
        },
      );
    }));
}

test("corpus generation has exact membership and survives unchanged inventory", () =>
  temporary((root) => {
    makeCorpus(fixture, root);
    const before = inventory(root, ["src", "package.json", "package-lock.json", "tsconfig.json"]);
    assert.equal(Object.keys(before).length, 1213);
    assert.deepEqual(
      inventory(root, ["src", "package.json", "package-lock.json", "tsconfig.json"]),
      before,
    );
    fs.appendFileSync(path.join(root, "src/cohort-00/Comp0.vue"), "\n");
    assert.notDeepEqual(
      inventory(root, ["src"])["src/cohort-00/Comp0.vue"],
      before["src/cohort-00/Comp0.vue"],
    );
    fs.writeFileSync(path.join(root, "src/extra.ts"), "export {};");
    assert.equal(Object.keys(inventory(root, ["src"])).length, 1211);
  }));

test("dependency inventory retains link identity and rejects external dependency links", () =>
  temporary((root) => {
    fs.writeFileSync(path.join(root, "entry"), "inside");
    fs.symlinkSync("entry", path.join(root, "internal"));
    assert.deepEqual(inventory(root, ["internal"]), { internal: { symlink: "entry" } });
    fs.symlinkSync(os.tmpdir(), path.join(root, "external"));
    assert.throws(() => inventory(root, ["external"]), /dependency link escapes consumer/u);
  }));

test("sample acceptance checks every raw byte, file order, diagnostic field and status", () => {
  const output = Buffer.from(
    '[{"file":"A","messages":[{"message":"x","severity":1}]},{"file":"B","messages":[]}]',
  );
  const process = { status: 1, signal: null, stdout: output, stderr: Buffer.alloc(0) };
  assert.equal(assertProcess(process, output), sha256(output));
  for (const change of [
    output.toString().replace('"x"', '"y"'),
    output.toString().replace('"severity":1', '"severity":2'),
    JSON.stringify(JSON.parse(output.toString()).reverse()),
    `${output.toString()}\n`,
  ])
    assert.throws(() => assertProcess({ ...process, stdout: Buffer.from(change) }, output));
  for (const change of [
    { status: 0 },
    { status: 2 },
    { signal: "SIGTERM" },
    { error: new Error("timeout") },
  ]) {
    assert.throws(() => assertProcess({ ...process, ...change }, output));
  }
});

function profile(side) {
  return {
    truncation: { dropped_spans: 0, dropped_counters: 0 },
    spans: structuredClone(frozen.authority.requestPhaseCounts[side]),
  };
}

test("entire31-phase vectors pin all probes and the1211-to2 snapshot transition", () => {
  for (const side of ["base", "head"]) {
    assertProfile(profile(side), frozen.authority.requestPhaseCounts[side]);
    assert.equal(profileCounts(profile(side)).length, 31);
  }
  const transport = (side) =>
    Object.fromEntries(
      profileCounts(profile(side))
        .filter((span) => span.key === "corsa.request.transport")
        .map((span) => [span.attribution.pass, span.count]),
    );
  assert.deepEqual(transport("base"), {
    initialize: 1,
    updateSnapshot: 1211,
    getDefaultProjectForFile: 1211,
    release: 1210,
    getSymbolsAtPositions: 1210,
    getTypesOfSymbols: 1210,
    typeToString: 7240,
    getPropertiesOfType: 4820,
    getSignaturesOfType: 2420,
    getReturnTypeOfSignature: 2420,
  });
  assert.deepEqual(transport("head"), {
    initialize: 1,
    updateSnapshot: 2,
    getDefaultProjectForFile: 2,
    release: 1,
    getSymbolsAtPositions: 1210,
    getTypesOfSymbols: 1210,
    typeToString: 7240,
    getPropertiesOfType: 4820,
    getSignaturesOfType: 2420,
    getReturnTypeOfSignature: 2420,
  });
});

test("missing duplicate unknown or truncated checker telemetry cannot pass", () => {
  const expected = frozen.authority.requestPhaseCounts.head;
  for (const mutate of [
    (data) => {
      data.truncation.dropped_spans = 1;
    },
    (data) => {
      data.truncation.dropped_counters = 1;
    },
    (data) => {
      data.spans.pop();
    },
    (data) => {
      data.spans.push(data.spans[0]);
    },
    (data) => {
      data.spans[0].count += 1;
    },
    (data) => {
      data.spans[0].attribution.pass = "other";
    },
    (data) => {
      data.spans[0].attribution.stage = "canon";
    },
  ]) {
    const data = profile("head");
    mutate(data);
    assert.throws(() => assertProfile(data, expected));
  }
});

test("measurement environment removes inherited instrumentation while keeping ordinary runtime variables", () => {
  assert.deepEqual(
    cleanEnvironment(
      {
        PATH: "/bin",
        HOME: "/consumer",
        VIZE_PROFILE: "1",
        CORSA_PATH: "old",
        TSGO_FLAGS: "x",
        NAPI_RS_FORCE_WASI: "1",
        LANG: "ja_JP",
        RAYON_NUM_THREADS: "1",
      },
      "/exact/tsc",
    ),
    {
      PATH: "/bin",
      HOME: "/consumer",
      LANG: "C.UTF-8",
      LC_ALL: "C.UTF-8",
      CORSA_PATH: "/exact/tsc",
    },
  );
});

test("RSS evidence must be one positive GNU-time observation", () => {
  assert.equal(parseRss("\tMaximum resident set size (kbytes): 1234\n"), 1234);
  for (const text of [
    "",
    "Maximum resident set size (kbytes): 0\n",
    "Maximum resident set size (kbytes): 1\nMaximum resident set size (kbytes): 2\n",
  ])
    assert.throws(() => parseRss(text));
  assert.equal(median([4, 1, 2]), 2);
  assert.equal(median([4, 1, 2, 3]), 2.5);
  for (const values of [[], [0], [-1], [Number.NaN]]) assert.throws(() => median(values));
});
