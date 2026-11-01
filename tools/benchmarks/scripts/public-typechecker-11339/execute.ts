import assert from "node:assert/strict";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, sep } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import {
  capture,
  healthy,
  recorder,
  retain,
  type Recorder,
  type Packet,
} from "../public-typechecker-11317/execute.ts";
import {
  cases,
  sources,
  sha256,
  originalFacts,
  limitations,
  type SourceFile,
  type Role,
  type Case,
} from "./contract.ts";
export { capture, healthy, recorder };
export type { Recorder };

export function inputImages(root: string, files: SourceFile[]) {
  assert(realpathSync(root) === root, "input root must be canonical");
  const directory = join(root, "src"),
    info = lstatSync(directory);
  assert(info.isDirectory() && !info.isSymbolicLink(), "source directory must be physical");
  assert(
    realpathSync(directory) === directory,
    "source directory must remain inside the canonical root",
  );
  const expected = files
    .filter((file) => file.path.startsWith("src/"))
    .map((file) => file.path.slice(4))
    .sort();
  assert(
    isDeepStrictEqual(readdirSync(join(root, "src")).sort(), expected),
    "unknown/missing source file",
  );
  return files.map((file) => {
    const path = join(root, file.path);
    assert(path.startsWith(root + sep), "input path must remain inside the canonical root");
    const stat = lstatSync(path);
    assert(
      stat.isFile() && !stat.isSymbolicLink(),
      "input must be a regular physical file: " + file.path,
    );
    assert(realpathSync(path) === path, "input path must be canonical: " + file.path);
    const raw = readFileSync(path);
    assert(
      raw.equals(Buffer.from(file.source)),
      "source bytes differ from the literal/authored input: " + file.path,
    );
    return {
      path: file.path,
      source: new TextDecoder("utf-8", { fatal: true }).decode(raw),
      bytes: raw.length,
      sha256: sha256(raw),
    };
  });
}
function apply(root: string, files: SourceFile[]) {
  rmSync(join(root, "src/Wrapped.vue"), { force: true });
  for (const file of files) {
    const path = join(root, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.source);
  }
}
export function guardedCapture(
  record: Recorder,
  label: string,
  argv: string[],
  root: string,
  verify: () => void,
  run = capture,
) {
  try {
    verify();
  } catch (error) {
    return retain(record, label, argv, root, new Date().toISOString(), {
      status: null,
      signal: null,
      stdout: null,
      stderr: null,
      error: error instanceof Error ? error : new Error(String(error)),
    });
  }
  return run(record, label, argv, root);
}
export function commandsFor(root: string, npx: string) {
  const vize = join(root, "node_modules/.bin/vize");
  return [
    { kind: "default-cli-text", argv: [vize, "check"] },
    { kind: "authored-cli-json-supplement", argv: [vize, "check", "-f", "json"] },
    {
      kind: "reported-independent-reference",
      argv: [process.execPath, npx, "vue-tsc", "--noEmit"],
    },
  ];
}
export function execute(
  root: string,
  role: Role,
  npx: string,
  original: SourceFile[],
  record: Recorder,
  verify: () => void,
) {
  let previous = sources(original, role, "Original");
  const results = cases.map((id) => {
    const files = sources(original, role, id);
    inputImages(root, previous);
    apply(root, files);
    const before = inputImages(root, files);
    record.json(id + ".inputs-before.json", before);
    const attempts = commandsFor(root, npx).map((command) => ({
      ...command,
      packet: guardedCapture(record, id + "." + command.kind, command.argv, root, () => {
        verify();
        inputImages(root, files);
      }),
    }));
    const after = inputImages(root, files);
    record.json(id + ".inputs-after.json", after);
    previous = files;
    return {
      id,
      sourceRole:
        id === "WrappedSupplement"
          ? "PR_AUTHORED_SUPPLEMENT_NOT_REPORTER_SOURCE"
          : "THREE_LITERAL_REPORTER_BODIES_WITH_NAMED_REPAIR_RESTORE",
      files,
      before,
      after,
      attempts,
    };
  });
  record.json("complete-observations.json", {
    role,
    results,
    limitations,
    publicAcceptance: false,
  });
  return results;
}
export type Result = ReturnType<typeof execute>;
export function factRows(parsed: unknown) {
  const dto = asRecord(parsed);
  assert.deepEqual(
    Object.keys(dto).sort(),
    ["files", "programs", "errorCount", "warningCount", "fileCount"].sort(),
  );
  return asArray(dto.files)
    .map((value) => asRecord(value))
    .flatMap((file) => {
      assert.deepEqual(Object.keys(file).sort(), ["diagnostics", "file"]);
      return asArray(file.diagnostics).map((value) => {
        const raw = asString(value),
          match = /^error:([0-9]+):([0-9]+) \[(TS[0-9]+)\] ([\s\S]+)$/u.exec(raw);
        assert(match, "unknown diagnostic format; preserve raw and refuse assessment");
        return {
          file: asString(file.file),
          line: Number(match[1]),
          column: Number(match[2]),
          code: match[3]!,
          message: match[4]!,
          raw,
        };
      });
    });
}
export function assessFacts(role: Role, id: Case, packet: Packet, files: SourceFile[]) {
  const dto: unknown = JSON.parse(packet.stdout ?? "");
  const value = asRecord(dto),
    rows = factRows(dto);
  const paths = files
    .filter((file) => file.path.startsWith("src/"))
    .map((file) => file.path)
    .sort();
  assert.deepEqual(
    asArray(value.files)
      .map((item) => asRecord(item))
      .map((file) => file.file),
    paths,
  );
  const config = files.find((file) => file.path === "tsconfig.json");
  assert(config);
  assert.deepEqual(value.programs, [
    {
      root: ".",
      tsconfig: "tsconfig.json",
      compilerOptions: asRecord(JSON.parse(config.source)).compilerOptions,
      files: paths,
    },
  ]);
  assert.equal(value.fileCount, paths.length);
  assert.equal(value.warningCount, 0);
  assert.equal(value.errorCount, rows.length);
  healthy(packet, rows.length ? 1 : 0);
  // No genuine public 0.635 observation exists yet; the report's unknown
  // external compiler cannot supply a missing-only BEFORE semantic oracle.
  if (role === "before")
    return { dto, rows, assessment: "PUBLIC_BEFORE_RETAINED_UNASSESSED", publicAcceptance: false };
  const expected =
    id === "Repair"
      ? []
      : [
          ...originalFacts,
          ...(id === "WrappedSupplement"
            ? [
                {
                  file: "src/Wrapped.vue",
                  line: 2,
                  column: 34,
                  code: "TS2339",
                  message: "Property 'toFixed' does not exist on type 'boolean'.",
                },
              ]
            : []),
        ];
  assert.equal(rows.length, expected.length, "unexpected/missing diagnostic; no count expansion");
  for (const [index, fact] of expected.entries()) {
    const row = rows[index]!;
    for (const key of ["file", "line", "column", "code"] as const)
      assert.equal(row[key], fact[key]);
    if (fact.message !== null) assert.equal(row.message, fact.message);
  }
  return {
    dto,
    rows,
    assessment: "PARTIAL_AUTHORED_POSITION_CODE_MESSAGE_FACTS",
    missingReceiverWholeMessageOwned: false,
    publicAcceptance: false,
  };
}
export function assess(role: Role, results: Result, record: Recorder) {
  const outcomes = results.map((result) => {
    assert.deepEqual(result.before, result.after, "authored input changed during command");
    const json = result.attempts.find((attempt) => attempt.kind === "authored-cli-json-supplement");
    assert(json);
    const facts = assessFacts(role, result.id, json.packet, result.files);
    // No empty-stderr assertion, stripping, fallback allowance or source-progress
    // normalization: preserve every whole stream for independent assessment.
    return {
      id: result.id,
      facts,
      entireAttempts: result.attempts,
      stderrAssessment: "WHOLE_RAW_RETAINED_NOT_YET_SOURCE_OWNED",
      referenceWholeStreamAssessed: false,
      textWholeStreamAssessed: false,
      wholeJsonOracleOwned: false,
      publicAcceptance: false,
    };
  });
  record.json("partial-semantic-facts.json", {
    role,
    outcomes,
    publicAcceptance: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
    exactReportedCompilerClaimed: false,
  });
  return outcomes;
}
