import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  comparePackets,
  readAssets,
  relocateVirtualPacket,
  type Row,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";
import { certifyCurrentProducerPopulation } from "../../tools/support/compat/content-mapper/producer-unmapped-ignores.ts";
import { certifyUnaffectedProjectionPackets } from "../../tools/support/compat/github/unicode-prop-projection-custody.mjs";
import {
  sourcePair,
  virtualPair,
  projectionRows,
  emitPath,
  ownerPath,
  producerPaths,
} from "./emit-overload-transition-fixture.ts";

test("the owned overload asset relocates the complete Unicode virtual packet", (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  const assets = readAssets(pair.base, pair.head, pair.sources);
  const packet = virtualPair(pair);
  const proof = relocateVirtualPacket(
    packet.base,
    packet.head,
    packet.source,
    assets,
    assets.headEmbedded,
  );
  assert.equal(proof.edits.length, 1);
  assert.equal(
    proof.edits[0].replacementSha256,
    createHash("sha256")
      .update(Buffer.concat([pair.after, pair.afterOwner]))
      .digest("hex"),
  );
});

test("owned emit inventory rejects every unauthenticated helper state", async (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  const assets = readAssets(pair.base, pair.head, pair.sources);
  const original = virtualPair(pair);
  const old = pair.before.toString(),
    next = pair.after.toString(),
    currentOwner = pair.afterOwner.toString();
  const mutations = {
    "missing current asset": (packet) => {
      packet.head.virtual_ts = packet.head.virtual_ts.replace(next, "");
    },
    "duplicate current asset": (packet) => {
      packet.head.virtual_ts += next;
    },
    "missing both assets with surviving owner": (packet) => {
      packet.base.virtual_ts = packet.base.virtual_ts.replace(old, "");
      packet.head.virtual_ts = packet.head.virtual_ts.replace(next, "");
    },
    "stale old alias": (packet) => {
      packet.head = structuredClone(packet.base);
    },
    "reversed alias transition": (packet) => {
      [packet.head, packet.base] = [packet.base, packet.head];
    },
    "mixed old and current aliases": (packet) => {
      packet.head.virtual_ts += old;
    },
    "foreign helper body": (packet) => {
      packet.head.virtual_ts = packet.head.virtual_ts.replace("TDepth", "UnknownDepth");
    },
    "moved owner boundary": (packet) => {
      packet.head.virtual_ts = packet.head.virtual_ts.replace(
        next + currentOwner,
        next + "\n" + currentOwner,
      );
    },
    "missing owner context": (packet) => {
      packet.head.virtual_ts = packet.head.virtual_ts.replace(currentOwner, "");
    },
  } satisfies Record<string, (packet: typeof original) => void>;
  for (const [name, mutate] of Object.entries(mutations))
    await context.test(name, () => {
      const packet = structuredClone(original);
      mutate(packet);
      assert.throws(() =>
        relocateVirtualPacket(packet.base, packet.head, packet.source, assets, assets.headEmbedded),
      );
    });
  await context.test("strict inventory precedes the unchanged packet shortcut", () => {
    const rows = Array.from({ length: 228 }, (_, index) => ({
      case: `case-${index}`,
      newline: "lf",
      target: "vue3",
      source: original.source,
      packet: original.base,
    })) as unknown as Row[];
    assert.throws(
      () => comparePackets(rows, structuredClone(rows), assets),
      /selected source transition/,
    );
  });
  await context.test("removing every owned helper cannot impersonate an absent population", () => {
    const rows = Array.from({ length: 228 }, (_, index) => ({
      case: `case-${index}`,
      newline: "lf",
      target: index < 192 ? "content-mapper" : "vue3",
      options: index < 192 ? index % 16 : undefined,
      source: original.source,
      packet: { text: "const unrelated = 1;", mappings: [], diagnostics: [] },
    }));
    assert.throws(
      () => comparePackets(rows, structuredClone(rows), assets),
      /all 152 owned emit packets/,
    );
  });
});

test("owned edits preserve all authored geometry and complete packet fields", async (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  const assets = readAssets(pair.base, pair.head, pair.sources);
  const original = virtualPair(pair);
  const mutations = {
    "crossing generated mapping": (packet) => {
      packet.base.virtual_ts_mappings[0].generated.start = 20;
    },
    "crossing generated subspan": (packet) => {
      packet.base.virtual_ts_mappings[0].subSpans[0].generated.start = 20;
    },
    "crossing semantic source": (packet) => {
      packet.base.virtual_ts_semantic_links[0].source.start = 20;
    },
    "crossing semantic target": (packet) => {
      packet.base.virtual_ts_semantic_links[0].target.start = 20;
    },
    "unrelated generated bytes": (packet) => {
      packet.head.virtual_ts += "// extra";
    },
    "changed diagnostic": (packet) => {
      packet.head.diagnostics[0].code++;
    },
    "changed opaque metadata": (packet) => {
      packet.head.opaque.producer = "different";
    },
  } satisfies Record<string, (packet: typeof original) => void>;
  for (const [name, mutate] of Object.entries(mutations))
    await context.test(name, () => {
      const packet = structuredClone(original);
      mutate(packet);
      assert.throws(() =>
        relocateVirtualPacket(packet.base, packet.head, packet.source, assets, assets.headEmbedded),
      );
    });
});

test("emit asset certificates bind physical producer images to selected full source SHAs", async (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  await context.test("missing selected source", () => {
    assert.throws(() => readAssets(pair.base, pair.head), /selected source revisions/);
  });
  await context.test("abbreviated source SHA", () => {
    assert.throws(
      () =>
        readAssets(pair.base, pair.head, { ...pair.sources, head: pair.sources.head.slice(0, 8) }),
      /full commit SHA/,
    );
  });
  await context.test("wrong selected source image", () => {
    assert.throws(
      () => readAssets(pair.base, pair.head, { ...pair.sources, head: pair.sources.base }),
      /uncertified emit producer source/,
    );
  });
  for (const file of [emitPath, ownerPath, ...producerPaths])
    await context.test(`changed physical image: ${file}`, () => {
      const selected = path.join(pair.head, file),
        original = readFileSync(selected);
      try {
        writeFileSync(selected, Buffer.concat([original, Buffer.from("\n// unauthenticated\n")]));
        assert.throws(() => readAssets(pair.base, pair.head, pair.sources));
      } finally {
        writeFileSync(selected, original);
      }
    });
});

test("the reused gap audit visits later current transforms even when the first is valid", () => {
  const valid = {
    text: "owned",
    mappings: [],
    diagnosticDirectives: {
      directives: [[0, 0, 0, 5, 0, 0]],
      unusedExpectDirectiveDiagnostics: [],
    },
  };
  const later = structuredClone(valid);
  later.diagnosticDirectives.directives = [];
  assert.equal(certifyCurrentProducerPopulation([valid, structuredClone(valid)]), 2);
  assert.throws(() => certifyCurrentProducerPopulation([valid, later]), /producer's unmapped gaps/);
});

test("unknown committed producer and owner sources cannot issue an asset certificate", async (context) => {
  for (const file of [ownerPath, ...producerPaths])
    await context.test(file, () => {
      const pair = sourcePair();
      try {
        const selected = path.join(pair.head, file);
        writeFileSync(
          selected,
          Buffer.concat([readFileSync(selected), Buffer.from("\n// different source\n")]),
        );
        pair.git("add", file);
        pair.git("commit", "--quiet", "-m", "test: retain an unqualified producer image");
        const sources = { ...pair.sources, head: pair.git("rev-parse", "HEAD") };
        assert.throws(
          () => readAssets(pair.base, pair.head, sources),
          /uncertified emit producer source/,
        );
      } finally {
        rmSync(pair.directory, { recursive: true, force: true });
      }
    });
});

test("the actual Unicode wrapper threads selected source custody into the owner certificate", (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  const packets = projectionRows(pair);
  assert.throws(
    () => certifyUnaffectedProjectionPackets(packets.base, packets.head, pair.base, pair.head),
    /selected source revisions/,
  );
  const proof = certifyUnaffectedProjectionPackets(
    packets.base,
    packets.head,
    pair.base,
    pair.head,
    pair.sources,
  );
  assert.equal(proof.completePackets, 228);
  assert.equal(proof.ownedEmitPackets, 152);
  assert.ok(proof.emitOverloads);
  assert.equal(proof.emitOverloads.head, pair.sources.head);
  assert.throws(
    () =>
      certifyUnaffectedProjectionPackets(packets.base, packets.head, pair.base, pair.head, {
        ...pair.sources,
        head: pair.sources.base,
      }),
    /uncertified emit producer source/,
  );
});

test("equal physical assets cannot bypass an authenticated selected source transition", async (context) => {
  const pair = sourcePair();
  context.after(() => rmSync(pair.directory, { recursive: true, force: true }));
  const unchanged = readAssets(pair.head, pair.head, { ...pair.sources, base: pair.sources.head });
  assert.equal(unchanged.emitOverloads, undefined);
  for (const [name, image] of [
    ["stale previous image", pair.before],
    ["premature current image", pair.after],
    ["foreign equal image", Buffer.from("type Foreign = unknown;\n")],
  ] as const)
    await context.test(name, () => {
      try {
        writeFileSync(path.join(pair.base, emitPath), image);
        writeFileSync(path.join(pair.head, emitPath), image);
        assert.throws(
          () => readAssets(pair.base, pair.head, pair.sources),
          /uncertified emit producer source/,
        );
      } finally {
        writeFileSync(path.join(pair.base, emitPath), pair.before);
        writeFileSync(path.join(pair.head, emitPath), pair.after);
      }
    });
});
