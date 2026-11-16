import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  aggregateProfileCounts,
  assertAggregatePackets,
  assertPacketAuthority,
  assertProcess,
  assertProfile,
  assertProfilePacket,
  assertShardProjects,
  assertShardPlan,
  cleanEnvironment,
  inventory,
  makeCorpus,
  makeShardPlan,
  median,
  optimizedShardProfile,
  parseResourceUsage,
  parseRss,
  profileCounts,
  readAuthority,
  sha256,
  shardPacket,
  writeShardProjects,
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

void test("three captured templates reproduce all121 source files and exact1210-file public output", () => {
  assert.equal(
    sha256(fs.readFileSync(path.join(fixture, "authority.json"))),
    "4b9eaf4c6a9a95f214b2aeb30892fd3d8ecadbc83b166760fec29f4c455db7dd",
  );
  assert.equal(frozen.seed.size, 121);
  assert.equal(frozen.rows.length, 1210);
  const diagnostics = frozen.rows.reduce((sum, row) => sum + row.messages.length, 0);
  assert.equal(diagnostics, 2820);
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

void test("ten deterministic shards retain every immutable source and complete packet", () => {
  const shards = makeShardPlan(frozen);
  assert.equal(
    sha256(JSON.stringify(shards)),
    "83ed0992694177b877044982a5604511d5372cc4fd41962c355cd3357abbc4ce",
  );
  assert.deepEqual(
    shards.map((shard) => shard.id),
    Array.from({ length: 10 }, (_, index) => `cohort-${String(index).padStart(2, "0")}`),
  );
  assert.deepEqual(
    shards.map((shard) => shard.packet.stdoutSha256),
    [
      "d03116438aed9778701f0d640991e13537a3f115cf1c3c64bffada4bd5899eef",
      "cc22c51c9aa67b3512e4b543b336b8ce59fa4e75520fb4c3f0bce6ae866f7c43",
      "7c10fd951108ec8b14c543cbc786b77f81f6385b4066d81d5c487940aa383fad",
      "1a42aabe5ac83a70258c85ed61d13d989a3289c87b231feacf4dd4d8324b5417",
      "c3f1aaee22efc5feeb9a4dab3b01feb33d6edbdffb878c27bfb72c00cd01bcbe",
      "0938956f9a447a0af61045c570d7977c5297c79a2e7d786f95bdad9a21439eb3",
      "4e0fad461aedafab6f4e60e173d73777ba0ff7daa33161cd0c686b34f71d58d2",
      "d95ad033d3372740837f7dcd925105f0a871b36fa3eec81efb033631450b88eb",
      "e758b95e3c84370b6aea0fa3903e2b6c6b53d79e7d6d299cbebd897ccf4dfc95",
      "ce4f68164a580a44c42496babfd0fd1a5dce9da1842097d17fabb9bd89fef9eb",
    ],
  );
  assert.equal(shards.flatMap((shard) => shard.members).length, 1210);
  assert.equal(new Set(shards.flatMap((shard) => shard.members)).size, 1210);
  for (const shard of shards) {
    assert.equal(shard.members.length, 121);
    assert.equal(shard.packet.fileCount, 121);
    assert.equal(shard.packet.diagnosticCount, 282);
  }
});

void test("missing duplicate foreign reordered and packet-drifted shards fail closed", () => {
  const original = makeShardPlan(frozen);
  for (const mutate of [
    (plan) => plan.pop(),
    (plan) => {
      plan[0].members[1] = plan[0].members[0];
    },
    (plan) => plan[0].members.push("src/cohort-00/foreign.ts"),
    (plan) => plan.reverse(),
    (plan) => {
      plan[0].packet.stdoutSha256 = "0".repeat(64);
    },
  ]) {
    const plan = structuredClone(original);
    mutate(plan);
    assert.throws(() => assertShardPlan(frozen, plan));
  }
});

void test("shard packets reconstruct the frozen aggregate and reject custody drift", () => {
  const shards = makeShardPlan(frozen);
  const packets = shards.map((shard) => ({
    id: shard.id,
    rows: shardPacket(frozen, shard).rows,
    ...shard.packet,
  }));
  const aggregate = assertAggregatePackets(frozen, shards, packets);
  assert.equal(aggregate.fileCount, 1210);
  assert.equal(aggregate.diagnosticCount, 2820);
  assert.equal(
    aggregate.stdoutSha256,
    "bf3a6089ca4026480ebcf5b188ae11f1e757c6067f59eea0fcb3ca121bcc50dc",
  );
  for (const mutate of [
    (rows) => rows.pop(),
    (rows) => rows.reverse(),
    (rows) => {
      rows[0].stdoutSha256 = "0".repeat(64);
    },
    (rows) => {
      rows[0].rows[0].messages = [];
    },
  ]) {
    const changed = structuredClone(packets);
    mutate(changed);
    assert.throws(() => assertAggregatePackets(frozen, shards, changed));
  }
});

void test("one run may seal current base drift while every later packet stays exact", () => {
  const shards = makeShardPlan(frozen);
  const authority = shards.map((shard) => {
    const rows = structuredClone(shardPacket(frozen, shard).rows);
    rows[0].messages[0].message = "current base behavior";
    const stdout = Buffer.from(JSON.stringify(rows, null, 2));
    return {
      id: shard.id,
      rows,
      fileCount: rows.length,
      diagnosticCount: rows.reduce((sum, row) => sum + row.messages.length, 0),
      byteLength: stdout.length,
      stdoutSha256: sha256(stdout),
    };
  });
  const aggregate = assertPacketAuthority(shards, authority, structuredClone(authority));
  assert.notEqual(aggregate.stdoutSha256, frozen.authority.expandedStdoutSha256);
  const changed = structuredClone(authority);
  changed[9].rows.at(-1).messages.at(-1).message = "head behavior drift";
  assert.throws(() => assertPacketAuthority(shards, authority, changed));
});

void test("shard projects isolate source membership without changing frozen authored bytes", () =>
  temporary((root) => {
    const generated = makeCorpus(fixture, root);
    const shards = makeShardPlan(generated);
    const projects = writeShardProjects(root, generated, shards);
    const configs = shards.map((shard) =>
      fs.readFileSync(path.join(root, shard.directory, "tsconfig.json"), "utf8"),
    );
    assert.equal(new Set(configs).size, 10);
    assert.deepEqual(JSON.parse(configs[0]).include, ["src/cohort-00/**/*"]);
    assert.deepEqual(
      projects.map((project) => project.sourceCount),
      Array.from({ length: 10 }, () => 121),
    );
    assert.equal(fs.existsSync(path.join(root, "src")), false);
    assert.equal(Object.keys(inventory(root, ["projects"])).length, 1220);
    assert.equal(
      sha256(fs.readFileSync(path.join(root, "projects/cohort-00/src/cohort-00/Comp0.vue"))),
      generated.authority.seedInputs["src/Comp0.vue"],
    );
  }));

void test("physical roots reject missing cross-shard config and local dependency drift", () => {
  const mutations = [
    (root) => fs.rmSync(path.join(root, "projects/cohort-00/src/cohort-00/Comp0.vue")),
    (root) => {
      const foreign = path.join(root, "projects/cohort-00/src/cohort-01/Comp0.vue");
      fs.mkdirSync(path.dirname(foreign), { recursive: true });
      fs.copyFileSync(path.join(root, "projects/cohort-01/src/cohort-01/Comp0.vue"), foreign);
    },
    (root) => {
      const config = path.join(root, "projects/cohort-00/tsconfig.json");
      const parsed = JSON.parse(fs.readFileSync(config, "utf8"));
      parsed.include = ["src/**/*"];
      fs.writeFileSync(config, `${JSON.stringify(parsed, null, 2)}\n`);
    },
    (root) => {
      const dependency = path.join(root, "projects/cohort-00/node_modules/foreign/package.json");
      fs.mkdirSync(path.dirname(dependency), { recursive: true });
      fs.writeFileSync(dependency, "{}");
    },
  ];
  for (const mutate of mutations) {
    temporary((root) => {
      const generated = makeCorpus(fixture, root);
      const shards = makeShardPlan(generated);
      writeShardProjects(root, generated, shards);
      mutate(root);
      assert.throws(() => assertShardProjects(root, generated, shards));
    });
  }
});

for (const name of ["component.vue.txt", "module.ts.txt", "cohort.stdout", "package-lock.json"]) {
  void test(`changed frozen ${name} fails before source generation`, () =>
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

void test("corpus generation has exact membership and survives unchanged inventory", () =>
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

void test("dependency inventory retains link identity and rejects external dependency links", () =>
  temporary((root) => {
    fs.writeFileSync(path.join(root, "entry"), "inside");
    fs.symlinkSync("entry", path.join(root, "internal"));
    assert.deepEqual(inventory(root, ["internal"]), { internal: { symlink: "entry" } });
    fs.symlinkSync(os.tmpdir(), path.join(root, "external"));
    assert.throws(() => inventory(root, ["external"]), /dependency link escapes consumer/u);
  }));

void test("sample acceptance checks every raw byte, file order, diagnostic field and status", () => {
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

void test("packet mismatch receipts stay bounded for early late and length drift", () => {
  const expected = Buffer.alloc(1024 * 1024, "a");
  const early = Buffer.from(expected);
  early[0] = "b".charCodeAt(0);
  const late = Buffer.from(expected);
  late[late.length - 1] = "b".charCodeAt(0);
  for (const actual of [
    early,
    late,
    expected.subarray(0, expected.length - 1),
    Buffer.concat([expected, Buffer.from("b")]),
  ]) {
    let error;
    try {
      assertProcess({ status: 1, signal: null, stdout: actual, stderr: Buffer.alloc(0) }, expected);
    } catch (caught) {
      error = caught;
    }
    assert.ok(error instanceof Error, "packet drift must throw");
    assert.ok(error.message.length < 2048, `bounded receipt was ${error.message.length} bytes`);
    assert.match(error.message, /"firstMismatch":\d+/u);
    assert.match(error.message, /"contextBase64":"[A-Za-z0-9+/=]*"/u);
  }
});

function profile(side) {
  return {
    truncation: { dropped_spans: 0, dropped_counters: 0 },
    spans: structuredClone(frozen.authority.requestPhaseCounts[side]),
  };
}

void test("entire31-phase vectors pin all probes and the1211-to2 snapshot transition", () => {
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

void test("optimized request vectors project exactly per shard and aggregate", () => {
  const perShard = optimizedShardProfile(frozen.authority);
  assert.equal(perShard.length, 31);
  const transport = Object.fromEntries(
    perShard
      .filter((span) => span.key === "corsa.request.transport")
      .map((span) => [span.attribution.pass, span.count]),
  );
  assert.deepEqual(transport, {
    getDefaultProjectForFile: 2,
    getPropertiesOfType: 482,
    getReturnTypeOfSignature: 242,
    getSignaturesOfType: 242,
    getSymbolsAtPositions: 121,
    getTypesOfSymbols: 121,
    initialize: 1,
    release: 1,
    typeToString: 724,
    updateSnapshot: 2,
  });
  const aggregate = aggregateProfileCounts(Array.from({ length: 10 }, () => perShard));
  const aggregateTransport = Object.fromEntries(
    aggregate
      .filter((span) => span.key === "corsa.request.transport")
      .map((span) => [span.attribution.pass, span.count]),
  );
  assert.deepEqual(aggregateTransport, {
    getDefaultProjectForFile: 20,
    getPropertiesOfType: 4820,
    getReturnTypeOfSignature: 2420,
    getSignaturesOfType: 2420,
    getSymbolsAtPositions: 1210,
    getTypesOfSymbols: 1210,
    initialize: 10,
    release: 10,
    typeToString: 7240,
    updateSnapshot: 20,
  });
});

void test("missing duplicate unknown or truncated checker telemetry cannot pass", () => {
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

void test("complete future telemetry retains explicit other while duplicate identities fail", () => {
  const current = profile("head");
  current.spans[0].count += 1;
  current.spans[0].attribution.pass = "other";
  const counts = assertProfilePacket(current);
  assert.equal(counts.length, 31);
  assert.equal(
    counts.some((row) => row.attribution.pass === "other"),
    true,
  );
  current.spans.push(structuredClone(current.spans[0]));
  assert.throws(() => assertProfilePacket(current));
});

void test("measurement environment removes inherited instrumentation while keeping ordinary runtime variables", () => {
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

void test("RSS evidence must be one positive GNU-time observation", () => {
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

void test("each shard resource packet carries CPU and peak RSS without sampled sums", () => {
  const packet = [
    "\tUser time (seconds): 1.25",
    "\tSystem time (seconds): 0.50",
    "\tMaximum resident set size (kbytes): 1234",
    "",
  ].join("\n");
  assert.deepEqual(parseResourceUsage(packet), {
    userCpuMs: 1250,
    systemCpuMs: 500,
    cpuMs: 1750,
    maximumRssKiB: 1234,
  });
  for (const changed of [
    packet.replace("1.25", "-1"),
    packet.replace("0.50", "missing"),
    packet.replace("1234", "0"),
    `${packet}\tUser time (seconds): 1.25\n`,
  ])
    assert.throws(() => parseResourceUsage(changed));
});
