const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const inputPath = path.resolve(
  __dirname,
  "../_fixtures/tooling/typescript-classification-runtime-11664/input.json",
);
const semantic = "encodedSemanticClassifications-full";
const syntactic = "encodedSyntacticClassifications-full";
const hash = (source) => crypto.createHash("sha256").update(source).digest("hex");

function createClassificationFixtures(workspace, output) {
  const inputBytes = fs.readFileSync(inputPath);
  const input = JSON.parse(inputBytes);
  assert.equal(input.schema, "vize.classificationLiveFixture.v1");
  assert.equal(input.issue, 11664);
  const directory = path.join(workspace, "src/分類-11664");
  fs.mkdirSync(directory, { recursive: true });
  const cases = input.cases.map((row) => {
    const prefix = `<script setup lang="ts">${row.newline}`;
    const suffix = `</script>${row.newline}<template />${row.newline}`;
    // Script setup variables are function-local in the native projection.
    const referencePrefix = `export {};${row.newline}function reference() {${row.newline}`;
    const referenceSuffix = `${row.newline}}${row.newline}`;
    const file = path.join(directory, row.id + row.extension);
    const references = Object.fromEntries(
      ["before", "after"].map((phase) => [
        phase,
        {
          file: path.join(directory, `${row.id}-${phase}.ts`),
          text: referencePrefix + row[phase] + referenceSuffix,
          range: { start: referencePrefix.length, length: row[phase].length },
        },
      ]),
    );
    fs.writeFileSync(file, prefix + row.before + suffix);
    for (const reference of Object.values(references))
      fs.writeFileSync(reference.file, reference.text);
    return { ...row, file, prefix, suffix, references };
  });
  const ordinary = { file: path.join(directory, "ordinary.ts"), text: input.ordinary };
  fs.writeFileSync(ordinary.file, ordinary.text);
  const fixture = { cases, ordinary, inputSha256: hash(inputBytes), output };
  fs.writeFileSync(
    path.join(output, "classification-inputs.json"),
    JSON.stringify(fixture, null, 2),
  );
  return fixture;
}

function triples(body, source, range = { start: 0, length: source.length }) {
  assert(
    Array.isArray(body.spans) && body.spans.length > 0 && body.spans.length % 3 === 0,
    "complete nonempty encoded triples required",
  );
  let previous = -1;
  const boundary = (at) =>
    !(at > 0 && source.charCodeAt(at) >= 0xdc00 && source.charCodeAt(at) <= 0xdfff);
  for (let at = 0; at < body.spans.length; at += 3) {
    const [start, length, classification] = body.spans.slice(at, at + 3);
    assert([start, length, classification].every(Number.isSafeInteger));
    assert(
      start >= previous && length > 0 && classification > 0,
      "ordered complete classifications required",
    );
    assert(
      start >= range.start && start + length <= range.start + range.length,
      "classification escaped the authored request",
    );
    assert(
      start >= 0 && start + length <= source.length && boundary(start) && boundary(start + length),
      "classification must use complete UTF-16 boundaries",
    );
    previous = start;
  }
  return body.spans;
}

function authoredOracle(reference, range, sourceStart) {
  // The TS syntactic API may include enclosing function tokens. Only the
  // independently declared reference script region belongs to this SFC.
  const result = [];
  for (let at = 0; at < reference.spans.length; at += 3) {
    const [start, length, classification] = reference.spans.slice(at, at + 3);
    const overlaps = start < range.start + range.length && start + length > range.start;
    if (!overlaps) continue;
    assert(
      start >= range.start && start + length <= range.start + range.length,
      "reference token crosses its fixed script boundary",
    );
    result.push(start + sourceStart - range.start, length, classification);
  }
  assert(result.length > 0);
  return { ...reference, spans: result };
}

function assertAuthoredClassification(actual, expected, source, range, completePacket) {
  triples(actual, source, range);
  assert.deepEqual(
    actual.spans,
    expected.spans,
    "whole authored triples, including type and modifier bits, must match",
  );
  if (completePacket)
    assert.deepEqual(actual, expected, "preserve the complete classification response body");
}

function observationRoles(commands) {
  const input = JSON.parse(fs.readFileSync(inputPath, "utf8"));
  const roles = commands.map(() => "ordinary-before");
  for (const inputCase of input.cases)
    for (const phase of ["before", "after"]) {
      for (const _command of commands)
        roles.push(`${inputCase.id}-${phase}-reference`, `${inputCase.id}-${phase}-sfc`);
      for (const _command of commands) roles.push(`ordinary-${inputCase.id}-${phase}`);
    }
  return roles;
}

async function checkClassifications(fixture, adapter) {
  const packet = {
    transport: adapter.transport,
    inputSha256: fixture.inputSha256,
    commands: adapter.commands,
    observations: [],
    comparisons: [],
  };
  const save = () =>
    fs.writeFileSync(
      path.join(fixture.output, "classification-results.json"),
      JSON.stringify(packet, null, 2),
    );
  const query = async (role, command, file, text, range) => {
    const response = await adapter.query(command, file, text, range);
    assert.equal(response.success, true, JSON.stringify(response));
    const observation = {
      role,
      command,
      file,
      source: text,
      sourceSha256: hash(text),
      range,
      response,
    };
    packet.observations.push(observation);
    save();
    triples(response.body, text);
    return response.body;
  };
  try {
    await adapter.open(fixture.ordinary.file, fixture.ordinary.text);
    const ordinary = {};
    for (const command of adapter.commands)
      ordinary[command] = await query(
        "ordinary-before",
        command,
        fixture.ordinary.file,
        fixture.ordinary.text,
        { start: 0, length: fixture.ordinary.text.length },
      );
    for (const input of fixture.cases) {
      for (const reference of Object.values(input.references))
        await adapter.open(reference.file, reference.text);
      const beforeText = input.prefix + input.before + input.suffix;
      await adapter.open(input.file, beforeText);
      const prior = {};
      for (const phase of ["before", "after"]) {
        const text = input.prefix + input[phase] + input.suffix;
        if (phase === "after") await adapter.change(input.file, beforeText, text);
        for (const command of adapter.commands) {
          const reference = input.references[phase];
          const oracleBody = await query(
            `${input.id}-${phase}-reference`,
            command,
            reference.file,
            reference.text,
            reference.range,
          );
          const expected = authoredOracle(oracleBody, reference.range, input.prefix.length);
          const actual = await query(`${input.id}-${phase}-sfc`, command, input.file, text, {
            start: 0,
            length: text.length,
          });
          assertAuthoredClassification(
            actual,
            expected,
            text,
            { start: input.prefix.length, length: input[phase].length },
            adapter.transport === "tsserver",
          );
          const unicode = text.indexOf("𐐀変数");
          assert(
            actual.spans.some(
              (start, index) =>
                index % 3 === 0 && start === unicode && actual.spans[index + 1] === 4,
            ),
            "the supplementary-plane identifier retains its UTF-16 length4",
          );
          if (phase === "after")
            assert.notDeepEqual(
              actual.spans,
              prior[command],
              "unsaved edits must change the complete authored classification packet",
            );
          else prior[command] = actual.spans;
          packet.comparisons.push({ id: input.id, phase, command, expected, actual });
          save();
        }
        for (const command of adapter.commands) {
          const control = await query(
            `ordinary-${input.id}-${phase}`,
            command,
            fixture.ordinary.file,
            fixture.ordinary.text,
            { start: 0, length: fixture.ordinary.text.length },
          );
          assert.deepEqual(
            control,
            ordinary[command],
            "SFC edits must preserve the complete ordinary TypeScript response",
          );
        }
      }
      assert.equal(
        fs.readFileSync(input.file, "utf8"),
        beforeText,
        "classification edits stay in the actual open buffer",
      );
    }
    packet.passed = true;
    return packet;
  } finally {
    save();
  }
}

module.exports = {
  createClassificationFixtures,
  checkClassifications,
  authoredOracle,
  triples,
  assertAuthoredClassification,
  observationRoles,
  semantic,
  syntactic,
};
