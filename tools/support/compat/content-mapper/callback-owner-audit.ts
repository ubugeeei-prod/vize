/** Audit complete captured commands against independently authored same-SDK inputs. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { FrameDecoder } from "../../../../tests/tooling/support/lsp/installed-alias-replay/frames.ts";
import {
  callbackOwnerProjects,
  callbackSetupReference,
  callbackSetupReferencePrefix,
  type SourceSpan,
} from "./callback-owner-projects.ts";

type Observation = {
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: unknown;
  stdout: string;
  stderr: string;
};
type Row = { id: string; files: Record<string, string>; observations: Record<string, Observation> };
type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
  continuation: string[];
};
const header = "<!-- 日本 🧭 -->";
const digest = (source: string) => createHash("sha256").update(source).digest("hex");

/** Every output line must belong to one diagnostic; no messages are filtered. */
function diagnostics(output: string): Diagnostic[] {
  const rows: Diagnostic[] = [];
  if (!output) return rows;
  assert.ok(output.endsWith("\n"));
  for (const line of output.slice(0, -1).split("\n")) {
    const match = /^(.*)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match) {
      rows.push({
        file: match[1],
        line: Number(match[2]),
        column: Number(match[3]),
        code: Number(match[4]),
        message: match[5],
        continuation: [],
      });
    } else {
      const previous = rows.at(-1);
      assert.ok(previous && /^\s+/u.test(line), `unclassified compiler output: ${line}`);
      previous.continuation.push(line);
    }
  }
  return rows;
}
/** The CLI names a binding only inside the first start tag of a line. */
function firstTagEnd(line: string): number {
  let quote = "";
  for (let index = 0; index < line.length; index++) {
    const character = line[index];
    if (quote) {
      if (character === quote) quote = "";
    } else if (character === '"' || character === "'") quote = character;
    else if (character === ">") return index + 1;
  }
  return line.length;
}
/** Each side names its own source build; only that producer identity may differ. */
function withoutMapperIdentity(stdout: string): string {
  return stdout.replaceAll(/vize@0\.0\.0-source-[0-9a-f]{40}/gu, "vize@<source>");
}
function processPacket(packet: Observation, statuses: number[]) {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.stderr, "");
  assert.ok(packet.status !== null && statuses.includes(packet.status));
}
function transforms(packet: Observation): Record<string, unknown>[] {
  processPacket(packet, [0]);
  const decoder = new FrameDecoder(),
    rows: Record<string, unknown>[] = [];
  decoder.push(Buffer.from(packet.stdout), (packet) => {
    assert.equal(packet.error, undefined);
    if (packet.id === 1)
      assert.deepEqual(packet.result, { diagnosticSource: "vize", positionEncoding: "utf-8" });
    if (Number(packet.id) >= 3 && Number(packet.id) < 100) {
      assert.ok(packet.result && typeof packet.result === "object");
      rows.push(packet.result as Record<string, unknown>);
    }
  });
  assert.equal(decoder.remaining, 0);
  assert.equal(rows.length, 2);
  return rows;
}
function point(source: string, start: number) {
  const lines = source.slice(0, start).split(/\r\n|\n|\r/u);
  return { line: lines.length, column: (lines.at(-1)?.length ?? 0) + 1 };
}
function at(
  span: SourceSpan,
  original: string,
  source: string,
  transform: (text: string) => string,
) {
  assert.equal(original.slice(span.start, span.end), span.text);
  const start = transform(original.slice(0, span.start)).length;
  const end = transform(original.slice(0, span.end)).length;
  assert.equal(source.slice(start, end), span.text);
  return { ...span, start, end, ...point(source, start) };
}
function mappingGeometry(result: Record<string, unknown>, source: string, spans: SourceSpan[]) {
  assert.equal(typeof result.text, "string");
  assert.ok(Array.isArray(result.mappings));
  const text = Buffer.from(result.text as string),
    authored = Buffer.from(source);
  const decoder = new TextDecoder("utf8", { fatal: true });
  for (const row of result.mappings as number[][]) {
    assert.equal(row.length, 6);
    assert.ok(row.every(Number.isSafeInteger));
    const [generated, generatedLength, original, originalLength] = row;
    assert.ok(generated >= 0 && original >= 0 && generatedLength >= 0 && originalLength >= 0);
    assert.ok(
      generated + generatedLength <= text.length && original + originalLength <= authored.length,
    );
    decoder.decode(text.subarray(generated, generated + generatedLength));
    decoder.decode(authored.subarray(original, original + originalLength));
  }
  for (const span of spans) {
    const start = Buffer.byteLength(source.slice(0, span.start));
    const length = Buffer.byteLength(source.slice(span.start, span.end));
    assert.ok(
      (result.mappings as number[][]).some(
        ([generated, generatedLength, original, originalLength]) => {
          return (
            original === start &&
            originalLength === length &&
            decoder.decode(text.subarray(generated, generated + generatedLength)) === span.text
          );
        },
      ),
      `complete exact callback value mapping: ${span.text}`,
    );
  }
}

export function auditCallbackOwners(directory: string) {
  const read = (file: string) => JSON.parse(fs.readFileSync(path.join(directory, file), "utf8"));
  const providers = read("providers.json"),
    inventory = read("inventory.json");
  assert.equal(providers.comparisonSourceSha, "78719553cf550bcffbf1d0fb891fda642f42de3a");
  assert.match(providers.sourceSha, /^[0-9a-f]{40}$/u);
  assert.equal(inventory.cases, 92);
  assert.equal(inventory.executedProcesses, 736);
  const rows = read("callback-cfa-observations.json") as Row[];
  assert.equal(rows.length, 92);
  const projects = callbackOwnerProjects(),
    outcomes: unknown[] = [];
  const planned = projects.flatMap((project) =>
    ["lf", "crlf"].flatMap((newline) =>
      ["plain", "unicode"].map((prefix) => `${project.id}-${newline}-${prefix}`),
    ),
  );
  assert.deepEqual(
    rows.map((row) => row.id),
    planned,
  );
  assert.deepEqual(inventory.ids, planned);
  let qualified = 0,
    preserved = 0,
    frontiers = 0,
    mapperFrontiers = 0;
  for (const row of rows) {
    const project = projects.find((project) => row.id.startsWith(`${project.id}-`));
    assert.ok(project);
    const newline =
      row.id.endsWith("-crlf-plain") || row.id.endsWith("-crlf-unicode") ? "\r\n" : "\n";
    const unicode = row.id.endsWith("-unicode");
    const parent = row.files["src/Min.vue"],
      original = project.inlineInputs["src/Min.vue"];
    const parentVariant = (source: string) =>
      source.replaceAll("  <Comp", `  ${unicode ? header : ""}<Comp`).replaceAll("\n", newline);
    assert.equal(parent, parentVariant(original));
    const referenceVariant = (source: string) =>
      `${unicode ? "// 日本 🧭\n" : ""}${callbackSetupReferencePrefix(source)}`.replaceAll(
        "\n",
        newline,
      );
    const reference =
      `${unicode ? "// 日本 🧭\n" : ""}${callbackSetupReference(project.reference.source)}`.replaceAll(
        "\n",
        newline,
      );
    const control = read(`projects/${row.id}/independent-reference/control.json`);
    assert.equal(control.source, reference);
    assert.equal(control.sourceSha256, digest(reference));
    assert.deepEqual(control.control, project);
    assert.equal(control.originalModuleSource, project.reference.source);
    assert.equal(control.originalModuleSourceSha256, digest(project.reference.source));
    processPacket(row.observations.reference, [0, 2]);
    processPacket(row.observations["native-reference"], [0, 1]);
    const expected = project.reference.expectedRoles;
    const actual = diagnostics(row.observations.reference.stdout);
    const nativeReference = diagnostics(row.observations["native-reference"].stdout);
    if (expected) {
      const spans = expected.map((role) => ({
        ...role,
        span: at(role.span, project.reference.source, reference, referenceVariant),
      }));
      assert.deepEqual(
        actual.map(({ file, line, column, code }) => ({ file, line, column, code })),
        spans.map(({ code, span }) => ({
          file: span.file,
          line: span.line,
          column: span.column,
          code,
        })),
        `${row.id}: complete handwritten reference vector`,
      );
      assert.equal(row.observations.reference.status, actual.length ? 2 : 0);
      assert.deepEqual(
        nativeReference.map(({ file, line, column, code }) => ({ file, line, column, code })),
        spans.map(({ code, span }) => ({
          file: span.file,
          line: span.line,
          column: span.column,
          code,
        })),
        `${row.id}: every physical 7.0.2 reference role`,
      );
      assert.equal(row.observations["native-reference"].status, nativeReference.length ? 1 : 0);
    }
    const before = transforms(row.observations["baseline-protocol"]);
    const candidate = transforms(row.observations["candidate-protocol"]);
    const bindings = project.authoredBindings.map(({ value }) =>
      at(value, original, parent, parentVariant),
    );
    mappingGeometry(candidate[1], parent, bindings);
    for (const side of ["baseline", "candidate"]) {
      processPacket(row.observations[`${side}-check`], [0, 1]);
      processPacket(row.observations[`${side}-mapper`], [0, 2]);
    }
    if (project.comparison === "complete-before-head-packets") {
      assert.deepEqual(
        candidate,
        before,
        `${row.id}: all unsupported transform fields stay identical`,
      );
      assert.equal(
        row.observations["candidate-check"].stdout,
        row.observations["baseline-check"].stdout,
      );
      assert.equal(
        row.observations["candidate-check"].status,
        row.observations["baseline-check"].status,
      );
      assert.equal(
        withoutMapperIdentity(row.observations["candidate-mapper"].stdout),
        withoutMapperIdentity(row.observations["baseline-mapper"].stdout),
      );
      assert.equal(
        row.observations["candidate-mapper"].status,
        row.observations["baseline-mapper"].status,
      );
      preserved++;
      outcomes.push({
        id: row.id,
        completeUnsupportedPreservation: true,
        referenceQualified: false,
      });
      continue;
    }
    if (project.qualification === "known-10739-frontier") {
      assert.equal(
        row.observations["candidate-check"].stdout,
        row.observations["baseline-check"].stdout,
        "canonical authored body errors and used EXPECT stay unchanged",
      );
      assert.equal(
        row.observations["candidate-check"].status,
        row.observations["baseline-check"].status,
      );
      frontiers++;
      outcomes.push({
        id: row.id,
        sourceQualified: false,
        issue: 10739,
        before: row.observations["baseline-mapper"],
        candidate: row.observations["candidate-mapper"],
      });
      continue;
    }
    assert.ok(expected && project.expected.diagnostics);
    const roles = [...project.expected.diagnostics];
    if (project.expected.expectation?.state === "unused")
      roles.unshift({
        role: "unused-expectation",
        code: 2578,
        span: project.expected.expectation.comment,
      });
    assert.equal(roles.length, actual.length);
    const authored = roles.map((role) => ({
      ...role,
      span: at(role.span, original, parent, parentVariant),
    }));
    const completeNative = authored.map((role, index) => {
      const diagnostic = nativeReference[index];
      assert.equal(diagnostic.code, role.code);
      const message =
        role.role === "unused-expectation"
          ? "Unused '@vue-expect-error' directive."
          : [diagnostic.message, ...diagnostic.continuation.map((line) => line.trimStart())].join(
              "\n",
            );
      const sourceLine = parent.split(/\r\n|\n/u).at(role.span.line - 1);
      assert.ok(sourceLine);
      const binding =
        role.role === "unused-expectation"
          ? "; binding: @vue-expect-error"
          : (role.role === "callback-assignment" || role.role === "callback-body") &&
              role.span.column - 1 < firstTagEnd(sourceLine)
            ? "; binding: 'pick'"
            : "";
      return `error:${role.span.line}:${role.span.column} [TS${role.code}] ${message} (source: ${sourceLine.trim()}${binding})`;
    });
    const config = JSON.parse(row.files["tsconfig.json"]),
      files = Object.keys(row.files)
        .filter((file) => file !== "tsconfig.json")
        .sort();
    const expectedNative = {
      files: files.map((file) => ({
        file,
        diagnostics: file === "src/Min.vue" ? completeNative : [],
      })),
      programs: [
        { root: ".", tsconfig: "tsconfig.json", compilerOptions: config.compilerOptions, files },
      ],
      errorCount: completeNative.length,
      warningCount: 0,
      fileCount: files.length,
    };
    const check = row.observations["candidate-check"],
      mapper = row.observations["candidate-mapper"];
    assert.deepEqual(
      JSON.parse(check.stdout),
      expectedNative,
      `${row.id}: every native field and authored diagnostic`,
    );
    assert.equal(check.status, completeNative.length ? 1 : 0);
    const expectedMapper = authored
      .map((role, index) => {
        if (role.role === "unused-expectation") {
          assert.ok(project.expected.expectation);
          const token = at(project.expected.expectation.token, original, parent, parentVariant);
          return `src/Min.vue(${token.line},${token.column}): error vize4: Unused '@vue-expect-error' directive\n`;
        }
        const diagnostic = actual[index];
        return `src/Min.vue(${role.span.line},${role.span.column}): error TS${role.code}: ${diagnostic.message}\n${diagnostic.continuation.map((line) => `${line}\n`).join("")}`;
      })
      .join("");
    if (project.mapperFrontier) {
      // The native packet is reference-exact. The mapper EXPECT interval is a
      // separate, pre-existing defect; it must keep its complete before packet.
      assert.notEqual(mapper.stdout, expectedMapper);
      assert.equal(
        withoutMapperIdentity(mapper.stdout),
        withoutMapperIdentity(row.observations["baseline-mapper"].stdout),
      );
      assert.equal(mapper.status, row.observations["baseline-mapper"].status);
      mapperFrontiers++;
      outcomes.push({
        id: row.id,
        completeNativePacket: true,
        mapperIssue: project.mapperFrontier,
      });
      continue;
    }
    assert.equal(mapper.stdout, expectedMapper, `${row.id}: complete same-SDK mapped stdout`);
    assert.equal(mapper.status, authored.length ? 2 : 0);
    qualified++;
    outcomes.push({ id: row.id, referenceQualified: true, completeNativeAndMapperPackets: true });
  }
  assert.equal(qualified, 36);
  assert.equal(mapperFrontiers, 4);
  assert.equal(preserved, 44);
  assert.equal(frontiers, 8);
  const report = {
    source: providers.sourceSha,
    before: providers.comparisonSourceSha,
    completeObservedProjects: rows.length,
    completeObservedCommands: inventory.executedProcesses,
    qualifiedReferenceProjects: qualified,
    unchangedUnsupportedProjects: preserved,
    explicitlyUnqualified10739Projects: frontiers,
    nativeOnly10871Projects: mapperFrontiers,
    wholeCorpusQualified: false,
    outcomes,
  };
  fs.writeFileSync(
    path.join(directory, "callback-owner-audit.json"),
    JSON.stringify(report, null, 2) + "\n",
  );
  return report;
}

if (
  process.argv[1] &&
  import.meta.url === new URL(`file://${path.resolve(process.argv[1])}`).href
) {
  assert.equal(process.argv.length, 3);
  console.log(JSON.stringify(auditCallbackOwners(path.resolve(process.argv[2]))));
}
