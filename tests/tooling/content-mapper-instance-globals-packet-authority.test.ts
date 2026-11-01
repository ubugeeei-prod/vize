import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type PacketGoldens,
} from "./support/content-mapper-instance-globals-10617.ts";
import {
  hasCompleteHostMatrix,
  hasCompletePacketQualification,
  selectPacketGoldens,
} from "./support/content-mapper-instance-globals-packets-10617.ts";

import { assertProfileComparatorNegatives } from "./support/content-mapper-instance-globals-profiles-10617.ts";

test(
  "profile controls reject hidden parsing, cache scope collapse and truncated evidence",
  assertProfileComparatorNegatives,
);

test("the original twenty provisional packets retain their complete immutable body", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const original = files.get("source-packets.json")!;
  assert.equal(sha(original), "bb2434fbf6d0f73165b5da6fc5674495c64727a4a0c19dcf09c01635d94d2cb9");
  const legacy = JSON.parse(original);
  assert.equal(legacy.complete, false);
  assert.equal(legacy.packets.length, 20);
  assert.equal(legacy.producer.sourceRevision, "125f93608ca1660ba68757e8ab0cac58b0451b2a");
  assert.deepEqual(
    selectPacketGoldens({ ...authority, currentSourcePackets: undefined }, files, cases),
    legacy,
  );
});

test("CURRENT cannot relabel the provisional capture or omit planned source coordinates", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const legacy = JSON.parse(files.get("source-packets.json")!);
  const name = "source-packets-current-125f93608ca1.json";
  const candidateFiles = new Map(files);
  const current = { ...authority, currentSourcePackets: name };
  assert.throws(() => selectPacketGoldens(current, candidateFiles, cases), /must be present/u);
  const raw = JSON.stringify(legacy);
  candidateFiles.set(name, raw);
  assert.throws(() => selectPacketGoldens(current, candidateFiles, cases), /authority row/u);
  const candidate = {
    ...current,
    files: [...authority.files, { path: name, bytes: Buffer.byteLength(raw), sha256: sha(raw) }],
  };
  assert.throws(() => selectPacketGoldens(candidate, candidateFiles, cases), /provisional/u);
  const incomplete = JSON.stringify({ ...legacy, complete: true });
  candidateFiles.set(name, incomplete);
  candidate.files[candidate.files.length - 1] = {
    path: name,
    bytes: Buffer.byteLength(incomplete),
    sha256: sha(incomplete),
  };
  assert.throws(
    () => selectPacketGoldens(candidate, candidateFiles, cases),
    /every planned packet/u,
  );
});

test("packet selection rejects foreign paths and byte changes outside its authority", () => {
  const { authority, files, cases } = readGlobalsFixture();
  assert.throws(() =>
    selectPacketGoldens(
      { ...authority, currentSourcePackets: "../source-packets.json" },
      files,
      cases,
    ),
  );
  const edited = new Map(files);
  edited.set("source-packets.json", `${edited.get("source-packets.json")} `);
  assert.throws(() =>
    selectPacketGoldens({ ...authority, currentSourcePackets: undefined }, edited, cases),
  );
});

test("complete packets cannot qualify omitted or skipped semantic and protocol checks", () => {
  const { files } = readGlobalsFixture();
  const provisional = JSON.parse(files.get("source-packets.json")!) as PacketGoldens;
  // Synthetic decision input only: the actual old fixture remains provisional.
  // The authority selector independently refuses its twenty rows as CURRENT.
  const golden = { ...provisional, complete: true };
  const sourceCases = golden.packets.map(({ label }: { label: string }, index: number) => ({
    id: index + 3,
    label,
    entry: { id: label.split("/")[1]! },
  }));
  const additionalHostIds = ["official-nightly", "pinned-source"];
  const outcomes = sourceCases.flatMap(({ id, label, entry }) =>
    [
      label,
      ...additionalHostIds.map((host) => `${label}/${host}`),
      `protocol/${id}/${entry.id}`,
      `whole-packet/${label}`,
    ].map((checkId) => ({
      id: checkId,
      outcome: "pass",
    })),
  );
  const controls = JSON.parse(files.get("receiver-plan-controls.json")!) as {
    programs: Array<{ id: string }>;
  };
  outcomes.push({ id: "receiver-plan/closed-protocol", outcome: "pass" });
  for (const { id } of controls.programs) {
    const prefix = `receiver-plan/${id}`;
    for (const suffix of [
      "frozen-independent-5.9",
      "symbols-and-geometry",
      "reported-beta",
      ...additionalHostIds,
    ])
      outcomes.push({ id: `${prefix}/${suffix}`, outcome: "pass" });
  }
  const baseline = { discoveryOnly: false, golden, sourceCases, additionalHostIds, outcomes };
  assert.equal(hasCompletePacketQualification(baseline), true);
  assert.equal(
    hasCompletePacketQualification({
      ...baseline,
      additionalHostIds: ["official-nightly"],
      outcomes: outcomes.filter(({ id }) => !id.endsWith("/pinned-source")),
    }),
    true,
    "generic two-host packets retain no matrix qualification credit",
  );
  assert.equal(hasCompletePacketQualification({ ...baseline, discoveryOnly: true }), false);
  assert.equal(hasCompletePacketQualification({ ...baseline, golden: provisional }), false);
  for (const outcome of ["discovery-unqualified", "skip", "gap"]) {
    const changed = outcomes.map((row, index) => (index === 0 ? { ...row, outcome } : row));
    assert.equal(hasCompletePacketQualification({ ...baseline, outcomes: changed }), false);
  }
  for (const prefix of [sourceCases[0]!.label, "protocol/", "whole-packet/", "receiver-plan/"]) {
    const missing = outcomes.filter(({ id }) => !id.startsWith(prefix));
    assert.equal(hasCompletePacketQualification({ ...baseline, outcomes: missing }), false);
  }
  assert.equal(
    hasCompletePacketQualification({ ...baseline, outcomes: [...outcomes, outcomes[0]!] }),
    false,
  );
  assert.equal(
    hasCompletePacketQualification({
      ...baseline,
      golden: { ...golden, packets: [...golden.packets].reverse() },
    }),
    false,
  );
});

test("only an admitted exact third-host roster acquires matrix qualification", () => {
  assert.equal(hasCompleteHostMatrix(true, true, ["official-nightly", "pinned-source"]), true);
  for (const ids of [
    [],
    ["official-nightly"],
    ["foreign", "pinned-source"],
    ["official-nightly", "foreign"],
    ["official-nightly", "pinned-source", "foreign"],
  ]) {
    assert.equal(hasCompleteHostMatrix(true, true, ids), false);
  }
  assert.equal(hasCompleteHostMatrix(false, true, ["official-nightly", "pinned-source"]), false);
  assert.equal(hasCompleteHostMatrix(true, false, ["official-nightly", "pinned-source"]), false);
});

test("the previous eighty SDK7 results remain bound to their immutable review", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-b66e208809ed.json" },
    files,
    cases,
  )!;
  assert.equal(golden.complete, true);
  assert.equal(golden.packets.length, 80);
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.producer.sourceRevision, "b66e208809edb527d3fd590835fa6295ba3285d1");
  assert.equal(provenance.capturedQualification.fullPacketQualification, false);
  assert.equal(provenance.capturedQualification.hostMatrixQualification, false);
  assert.equal(provenance.strictQualificationClaim, false);
  assert.equal(
    provenance.independentCurrentReview.rawSourceProtocolSha256,
    "eb78ab15eb122789df7a6b5778eac61fe99274edae7d87dd5d30fe2702fccf21",
  );
  assert.deepEqual(provenance.independentNativeTuplesReview.totals, {
    nativeMain: 240,
    receiverNativeAndActorCli: 12,
    optionsMethodFormerDiscoveryFullyChecked: 6,
  });
  assert.deepEqual(provenance.primaryNativeReview.semanticFailures, []);
  assert.deepEqual(provenance.primaryNativeReview.nonWholePacketGaps, []);
  const reviewed = provenance.independentCurrentReview.wholeTransformReview.rows as Array<{
    id: number;
    allFieldSerializationSha256: string;
    textSha256: string;
  }>;
  assert.equal(reviewed.length, 80);
  for (const [index, packet] of golden.packets.entries()) {
    const entry = cases[index % 40]!;
    const source =
      index < 40
        ? entry.source
        : entry.source.replace("{{ ", '{{ "日本語🍣" + ').replaceAll("\n", "\r\n");
    assert.equal(packet.sourceSha256, sha(source), packet.label);
    assert.equal(packet.globalsSha256, sha(entry.globals), packet.label);
    assert.equal(reviewed[index]!.id, index + 3);
    assert.equal(sha(JSON.stringify(packet.result)), reviewed[index]!.allFieldSerializationSha256);
    assert.equal(sha(packet.result.text), reviewed[index]!.textSha256);
  }
});

test("the previous SDK9 population retains all eighty actual packets without captured qualification credit", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-4543862023a0.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-b66e208809ed.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(golden.complete, true);
  assert.equal(golden.packets.length, 80);
  assert.equal(provenance.producer.sourceRevision, "4543862023a083a299c32761f702b2a6717c56e3");
  assert.equal(provenance.producer.sourceTree, "0f1122911d3b9bba9077338256d516e8d9e59f9f");
  assert.equal(provenance.producer.sourcePackageDeclarationFiles, 9);
  assert.equal(provenance.sourceAdapter.helperFiles.length, 9);
  assert.equal(provenance.capturedQualification.status, "unqualified");
  assert.equal(provenance.capturedQualification.fullPacketQualification, false);
  assert.equal(provenance.capturedQualification.hostMatrixQualification, false);
  assert.equal(provenance.strictQualificationClaim, false);
  assert.equal(provenance.nativeAcceptanceCredit, false);
  const changed = [
    "lf/slot-shadow",
    "lf/mixed-slot-scope",
    "crlf-unicode/slot-shadow",
    "crlf-unicode/mixed-slot-scope",
  ];
  const outcomes = provenance.capturedQualification.outcomes as Array<{
    id: string;
    outcome: string;
  }>;
  assert.equal(outcomes.length, 499);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    changed.map((label) => `whole-packet/${label}`),
  );
  const inputs = provenance.sourceInputs as Array<{
    label: string;
    utf8: string;
    sha256: string;
  }>;
  assert.equal(inputs.length, 80);
  const observedChanges: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const entry = cases[index % 40]!;
    const source =
      index < 40
        ? entry.source
        : entry.source.replace("{{ ", '{{ "日本語🍣" + ').replaceAll("\n", "\r\n");
    assert.equal(packet.sourceSha256, sha(source));
    assert.equal(packet.globalsSha256, sha(entry.globals));
    assert.deepEqual(inputs[index], {
      label: packet.label,
      bytes: Buffer.byteLength(source),
      sha256: sha(source),
      utf8: source,
    });
    const old = previous.packets[index]!.result;
    if (JSON.stringify(old) === JSON.stringify(packet.result)) continue;
    observedChanges.push(packet.label);
    const calls = [
      ...old.text.matchAll(
        /import\("vize\/virtual-ts"\)\.\$SlotPayload<typeof __vize_slot_(props_)?host_([0-9]+), "default">/gu,
      ),
    ];
    assert.equal(calls.length, 2);
    assert.deepEqual(
      calls.map((call) => call[1] ?? ""),
      ["", "props_"],
    );
    assert.equal(calls[0]![2], calls[1]![2]);
    const before = Buffer.from(old.text),
      after = Buffer.from(packet.result.text),
      insertion = Buffer.from(", typeof Child"),
      positions = calls.map(
        (call) => Buffer.byteLength(old.text.slice(0, call.index)) + Buffer.byteLength(call[0]) - 1,
      );
    let text = before;
    for (const position of positions.toReversed())
      text = Buffer.concat([text.subarray(0, position), insertion, text.subarray(position)]);
    assert.equal(text.equals(after), true, "entire text has only two source-owned arguments");
    const point = (value: number) =>
      value + positions.filter((position) => position <= value).length * insertion.length;
    const mappings = old.mappings.map((mapping) => {
      const start = mapping[0]!,
        end = start + mapping[1]!;
      for (const position of positions)
        assert.ok(
          end <= position || start >= position,
          "mapped content cannot overlap scaffolding",
        );
      const nextStart = point(start),
        nextEnd = point(end);
      assert.equal(before.subarray(start, end).equals(after.subarray(nextStart, nextEnd)), true);
      return [nextStart, nextEnd - nextStart, ...mapping.slice(2)];
    });
    assert.equal(
      JSON.stringify({ ...old, text: packet.result.text, mappings }),
      JSON.stringify(packet.result),
      "all opaque fields, diagnostics, directives, semantic links and authored ranges remain exact",
    );
  }
  assert.deepEqual(observedChanges, changed);
});

test("CURRENT resolves the complete authenticated population in authored case order", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(authority, files, cases)!;
  assert.equal(golden.complete, true);
  assert.equal(golden.packets.length, 80);
  const producer = golden.producer as { sourceRevision: string };
  assert.equal(
    authority.currentSourcePackets,
    `source-packets-current-${producer.sourceRevision.slice(0, 12)}.json`,
  );
  assert.ok(files.has(golden.provenance as string));
  for (const [index, packet] of golden.packets.entries()) {
    const entry = cases[index % 40]!;
    const source =
      index < 40
        ? entry.source
        : entry.source.replace("{{ ", '{{ "日本語🍣" + ').replaceAll("\n", "\r\n");
    assert.equal(packet.label, `${index < 40 ? "lf" : "crlf-unicode"}/${entry.id}`);
    assert.equal(packet.sourceSha256, sha(source));
    assert.equal(packet.globalsSha256, sha(entry.globals));
  }
});

test("the observed comment-offset population preserves whole results and failed capture credit", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const golden = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-d0f4e1da7022.json" },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: "source-packets-current-4543862023a0.json" },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.source.sourceRevision, "d0f4e1da7022399172ba9a414ce90bdd7fc00145");
  assert.equal(provenance.source.sourceTree, "64bb277f63ba3152cae051595b7500eb4c6d9f3f");
  assert.equal(provenance.sourceLaunch.receipt.sourceRevision, provenance.source.sourceRevision);
  assert.equal(provenance.sourceLaunch.receipt.binarySha256, provenance.source.binarySha256);
  assert.equal(provenance.actualQualification.status, "unqualified");
  assert.equal(provenance.actualQualification.fullPacketQualification, false);
  assert.equal(provenance.actualQualification.hostMatrixQualification, false);
  assert.equal(provenance.limits.publicAcceptance, false);
  assert.equal(provenance.limits.newNativeExecution, false);
  assert.deepEqual(provenance.actualQualification.outcomes, provenance.actualOutcomes);
  const changed = ["lf/string-comment-role", "crlf-unicode/string-comment-role"];
  const outcomes = provenance.actualOutcomes as Array<{ id: string; outcome: string }>;
  assert.equal(outcomes.length, 398);
  assert.deepEqual(
    outcomes.filter((row) => row.outcome !== "pass").map((row) => row.id),
    changed.map((label) => `whole-packet/${label}`),
  );
  assert.equal(provenance.actualInputs.length, 80);
  const observed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const entry = cases[index % 40]!;
    const source =
      index < 40
        ? entry.source
        : entry.source.replace("{{ ", '{{ "日本語🍣" + ').replaceAll("\n", "\r\n");
    assert.equal(packet.sourceSha256, sha(source));
    assert.equal(packet.globalsSha256, sha(entry.globals));
    assert.deepEqual(provenance.actualInputs[index], {
      label: packet.label,
      bytes: Buffer.byteLength(source),
      sha256: sha(source),
    });
    const old = previous.packets[index]!.result;
    if (JSON.stringify(old) === JSON.stringify(packet.result)) continue;
    observed.push(packet.label);
    assert.equal(
      JSON.stringify({ ...old, mappings: packet.result.mappings }),
      JSON.stringify(packet.result),
      "every nonmapping field and its order stays unchanged",
    );
    assert.deepEqual(packet.result.mappings.slice(0, -2), old.mappings.slice(0, -1));
    const authored = Buffer.from(source),
      generated = Buffer.from(packet.result.text);
    const known = Buffer.from(".known "),
      tail = Buffer.from(' + "$ghost"');
    const a = authored.indexOf(known),
      b = authored.indexOf(tail);
    const x = generated.indexOf(known),
      y = generated.indexOf(tail);
    for (const start of [a, b, x, y]) assert.ok(start >= 0);
    assert.equal(authored.subarray(a + known.length, b).toString(), "/* $ghost */");
    assert.deepEqual(packet.result.mappings.slice(-2), [
      [x, known.length, a, known.length, 0, 1048575],
      [y, tail.length, b, tail.length, 0, 1048575],
    ]);
    const proof = provenance.wholeComparison.changedResults.find(
      (row: { label: string }) => row.label === packet.label,
    );
    assert.equal(proof.wholeSource, source);
    assert.deepEqual(proof.previousMappings, old.mappings);
    assert.deepEqual(proof.currentMappings, packet.result.mappings);
  }
  assert.deepEqual(observed, changed);
  assert.equal(provenance.wholeComparison.unchangedWholePackets, 78);
});
