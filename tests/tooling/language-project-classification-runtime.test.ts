import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import {
  CLASSIFICATION_RUNTIME_ROOT as directory,
  CLASSIFICATION_RUNTIME_ID,
  authenticateClassificationRuntime,
  historicalClassificationConsumers,
} from "../../tools/support/compat/github/language-project-classification-runtime.ts";

const root = path.resolve(import.meta.dirname, "../..");
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const read = (_side: "base" | "head", file: string) => {
  const source = path.join(root, file);
  return fs.existsSync(source) ? fs.readFileSync(source) : null;
};
const custodyPath = directory + "custody.json";
const record = () => JSON.parse(read("head", custodyPath)!.toString());
const require = createRequire(import.meta.url);
const probe = require("../typescript-plugin/classification-probe.cjs");
const provider = require("../../editors/vscode/test/suite/classification-provider.cjs");

test("live runtime successors preserve six original consumers and leave base and unrelated reads actual", () => {
  const metadata = record();
  assert.equal(metadata.id, CLASSIFICATION_RUNTIME_ID);
  const originals = authenticateClassificationRuntime(read);
  const historical = historicalClassificationConsumers(read);
  assert.equal(originals.size, 6);
  for (const row of metadata.transitions) {
    const original = gunzipSync(read("head", row.gzip.path)!);
    assert.equal(digest(original), row.before.sha256);
    assert.deepEqual(originals.get(row.before.path), original);
    assert.deepEqual(historical("head", row.before.path), original);
    assert.deepEqual(historical("base", row.before.path), read("base", row.before.path));
    assert.notDeepEqual(original, read("head", row.after.path));
    const beforeRead = (side: "base" | "head", file: string) =>
      file === row.before.path ? original : read(side, file);
    assert.deepEqual(
      historicalClassificationConsumers(beforeRead)("head", row.before.path),
      original,
    );
  }
  assert.deepEqual(
    historical("head", "npm/typescript-plugin/index.cjs"),
    read("head", "npm/typescript-plugin/index.cjs"),
  );
  assert.equal(historical("head", "unknown-future.cjs"), null);
});

test("missing, changed, future and arbitrarily repinned runtime members cannot obtain archived authority", () => {
  const metadata = record();
  const members = [
    custodyPath,
    metadata.predecessor.path,
    metadata.input.path,
    ...metadata.consumers.map((row: any) => row.path),
    ...metadata.transitions.flatMap((row: any) => [row.after.path, row.gzip.path]),
  ];
  for (const target of members)
    for (const missing of [false, true]) {
      const altered = (side: "base" | "head", file: string) =>
        file === target
          ? missing
            ? null
            : Buffer.concat([read(side, file)!, Buffer.from("\nfuture mutation")])
          : read(side, file);
      assert.throws(
        () => authenticateClassificationRuntime(altered),
        /classification.*runtime/,
        target,
      );
      assert.throws(
        () =>
          historicalClassificationConsumers(altered)("head", metadata.transitions[0].after.path),
        /classification.*runtime/,
        target,
      );
    }
  for (const mutate of [
    (row: any) => {
      row.id = "unregistered";
    },
    (row: any) => {
      row.id = [CLASSIFICATION_RUNTIME_ID, CLASSIFICATION_RUNTIME_ID];
    },
    (row: any) => {
      row.transitions.pop();
    },
    (row: any) => {
      row.transitions.push(row.transitions[0]);
    },
    (row: any) => {
      row.transitions[0].after.sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.consumers[0].sha256 = "f".repeat(64);
    },
    (row: any) => {
      row.extra = "future";
    },
  ]) {
    const changed = structuredClone(metadata);
    mutate(changed);
    assert.throws(
      () =>
        authenticateClassificationRuntime((side, file) =>
          file === custodyPath ? Buffer.from(JSON.stringify(changed)) : read(side, file),
        ),
      /custody missing or changed/,
    );
  }
});

test("new helper and input paths select the existing jobs and cannot be removed alone", () => {
  const triggers = [
    [
      ".github/workflows/typescript-plugin.yml",
      ["tests/_fixtures/tooling/typescript-classification-runtime-11664/**"],
    ],
    [
      ".github/workflows/typescript-provider-vsix.yml",
      [
        "tests/typescript-plugin/classification-*",
        "tests/_fixtures/tooling/typescript-classification-runtime-11664/**",
        "editors/vscode/test/suite/classification-provider.cjs",
      ],
    ],
    [
      ".github/workflows/language-project-fixtures.yml",
      [
        "tests/_fixtures/tooling/typescript-classification-*/**",
        "editors/vscode/test/suite/classification-provider.cjs",
      ],
    ],
  ] as const;
  const metadata = record();
  for (const [workflow, paths] of triggers) {
    const current = read("head", workflow)!.toString();
    for (const trigger of paths) {
      const line = `      - ${trigger}\n`;
      assert.equal(current.split(line).length, 2);
      const altered = (side: "base" | "head", file: string) =>
        side === "head" && file === workflow
          ? Buffer.from(current.replace(line, ""))
          : read(side, file);
      assert.throws(
        () =>
          historicalClassificationConsumers(altered)("head", metadata.transitions[0].after.path),
        /unreviewed producer registration/,
      );
    }
    if (workflow !== ".github/workflows/language-project-fixtures.yml") {
      const transition = metadata.transitions.find((row: any) => row.after.path === workflow);
      const before = gunzipSync(read("head", transition.gzip.path)!).toString();
      assert.equal(
        current.slice(current.indexOf("\njobs:")),
        before.slice(before.indexOf("\njobs:")),
        "packed and installed job bodies remain unchanged",
      );
    }
  }
});

test("whole authored packets reject empty, reordered, mistyped, modifier-losing and escaping triples", () => {
  const source = "a 𐐀変数 b";
  const expected = { spans: [0, 1, 2049, 2, 4, 2081, 7, 1, 2048], endOfLineState: 0 };
  probe.assertAuthoredClassification(
    expected,
    expected,
    source,
    { start: 0, length: source.length },
    true,
  );
  for (const spans of [
    [],
    [7, 1, 2048, 2, 4, 2081, 0, 1, 2049],
    [0, 1, 257, 2, 4, 2081, 7, 1, 2048],
    [0, 1, 2049, 2, 4, 2049, 7, 1, 2048],
    [0, 1, 2049, 3, 3, 2081, 7, 1, 2048],
    [0, 1, 2049, 2, 4, 2081, 8, 1, 2048],
  ])
    assert.throws(() =>
      probe.assertAuthoredClassification(
        { ...expected, spans },
        expected,
        source,
        { start: 0, length: source.length },
        true,
      ),
    );
  assert.throws(() =>
    probe.assertAuthoredClassification(
      { ...expected, endOfLineState: 1 },
      expected,
      source,
      { start: 0, length: source.length },
      true,
    ),
  );
  assert.throws(() => probe.triples(expected, source, { start: 2, length: 4 }));
});

class Position {
  readonly line: number;
  readonly character: number;
  constructor(line: number, character: number) {
    this.line = line;
    this.character = character;
  }
}
function document(source: string) {
  const lines = source.split("\n");
  const starts = lines.map((_line, index) =>
    lines.slice(0, index).reduce((size, line) => size + line.length + 1, 0),
  );
  return {
    getText: () => source,
    offsetAt: (position: Position) => {
      assert(position instanceof Position, "Invalid argument: expected Position");
      return (starts[position.line] ?? source.length) + position.character;
    },
    positionAt: (offset: number) => {
      const line = starts.findLastIndex((start) => start <= offset);
      return new Position(line, offset - starts[line]!);
    },
  };
}
const legend = {
  tokenTypes: [
    "class",
    "enum",
    "interface",
    "namespace",
    "typeParameter",
    "type",
    "parameter",
    "variable",
    "enumMember",
    "property",
    "function",
    "method",
  ],
  tokenModifiers: ["declaration", "static", "async", "readonly", "defaultLibrary", "local"],
};

test("the GUI provider decoder preserves ordered UTF16 triples and the complete type/modifier legend", () => {
  const doc = document("a\r\n𐐀変数");
  const value = { data: [0, 0, 1, 7, 1, 1, 0, 4, 7, 33] };
  assert.deepEqual(
    provider.decodeTokens(doc, value, legend, Position).spans,
    [0, 1, 2049, 3, 4, 2081],
  );
  for (const data of [
    [],
    [0, 0, 1, 7],
    [0, 0, 1, 12, 1],
    [0, 0, 1, 7, 256],
    [1, 1, 3, 7, 33],
    [-1, 0, 1, 7, 1],
    [4, 0, 1, 7, 1],
  ])
    assert.throws(() => provider.decodeTokens(doc, { data }, legend, Position));
  assert.throws(() =>
    provider.decodeTokens(
      doc,
      value,
      { ...legend, tokenTypes: [...legend.tokenTypes].reverse() },
      Position,
    ),
  );
  assert.throws(() =>
    provider.decodeTokens(
      doc,
      value,
      {
        ...legend,
        tokenModifiers: [...legend.tokenModifiers].reverse(),
      },
      Position,
    ),
  );
});

test("GUI qualification requires every actual encoded request and distinct complete server response", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "classification-log-"));
  try {
    const file = path.join(temporary, "source.ts");
    fs.writeFileSync(file, "a");
    const roles = probe.observationRoles([probe.semantic]);
    const observations = roles.map((role: string) => ({
      role,
      command: probe.semantic,
      file,
      range: { start: 0, length: 1 },
      response: { body: { spans: [0, 1, 2049] } },
    }));
    const qualification = {
      transport: "vscode-built-in-range-semantic-provider",
      commands: [probe.semantic],
      passed: true,
      observations,
    };
    const packets = observations.flatMap((_row: any, index: number) => [
      {
        type: "request",
        seq: index + 1,
        command: probe.semantic,
        arguments: { file, start: 0, length: 1, format: "2020" },
      },
      {
        type: "response",
        request_seq: index + 1,
        command: probe.semantic,
        success: true,
        body: { spans: [0, 1, 2049], endOfLineState: 0 },
      },
    ]);
    const log = path.join(temporary, "tsserver.log");
    const expectedProvider = { source: "a".repeat(40), nativeBuildToken: "b".repeat(32) };
    const write = (rows: object[]) =>
      fs.writeFileSync(
        log,
        `Info 0 [12:00:00.000] [vize] TypeScript native provider ${JSON.stringify({ origin: "bundled", ...expectedProvider })}\n` +
          rows
            .map(
              (packet: any, index) =>
                `Info ${index} [12:00:00.000] ${packet.type}:\n${JSON.stringify(packet, null, 2)}\n`,
            )
            .join(""),
      );
    write(packets);
    assert.deepEqual(provider.logPackets(fs.readFileSync(log, "utf8")), packets);
    assert.equal(
      provider.verifyClassificationLogs([log], qualification, expectedProvider).observations.length,
      observations.length,
    );
    assert.equal(
      provider.verifyClassificationLogs([log], qualification, expectedProvider).syntaxExecuted,
      false,
    );
    for (const mutate of [
      (rows: any[]) => {
        rows.splice(-2);
      },
      (rows: any[]) => {
        rows[0].command = "quickinfo";
      },
      (rows: any[]) => {
        rows[0].arguments.length++;
      },
      (rows: any[]) => {
        rows[0].arguments.format = "original";
      },
      (rows: any[]) => {
        rows[0].arguments.file = log;
      },
      (rows: any[]) => {
        rows[1].success = false;
      },
      (rows: any[]) => {
        rows[1].command = "quickinfo";
      },
      (rows: any[]) => {
        rows[1].body.spans[2]++;
      },
    ]) {
      const changed = structuredClone(packets);
      mutate(changed);
      write(changed);
      assert.throws(
        () => provider.verifyClassificationLogs([log], qualification, expectedProvider),
        /missing actual encoded/,
      );
    }
    write(packets);
    assert.throws(
      () =>
        provider.verifyClassificationLogs([log], qualification, {
          ...expectedProvider,
          source: "c".repeat(40),
        }),
      /responding server/,
    );
    assert.throws(
      () =>
        provider.verifyClassificationLogs([log], qualification, {
          ...expectedProvider,
          nativeBuildToken: "c".repeat(32),
        }),
      /responding server/,
    );
    fs.writeFileSync(log, fs.readFileSync(log, "utf8").split("\n").slice(1).join("\n"));
    assert.throws(
      () => provider.verifyClassificationLogs([log], qualification, expectedProvider),
      /responding server/,
    );
    write(packets);
    const omitted = structuredClone(qualification);
    omitted.observations.pop();
    assert.throws(
      () => provider.verifyClassificationLogs([log], omitted, expectedProvider),
      /every before\/after/,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
