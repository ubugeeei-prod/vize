import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  input,
  sources,
  originalFacts,
  type Case,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/contract.ts";
import {
  assessFacts,
  recorder,
} from "../../tools/benchmarks/scripts/public-typechecker-11339/execute.ts";
import { retain } from "../../tools/benchmarks/scripts/public-typechecker-11317/execute.ts";

// Independently specified frozen input contexts, not copied from an observed
// compiler response or computed by the implementation under test.
const originalLine =
  '<p :title="greet(1)">{{ isPC.toFixed() }} {{ currentDevice.toFixed() }} {{ fromBase.toUpperCase() }} {{ missing }}</p>';
const wrappedLine =
  "<p :title=\"greet('x')\">{{ isPC.toFixed() }} {{ count.toFixed() }} {{ fromBase.toFixed() }}</p>";
const context = ` (source: ${originalLine})`;
const titleContext = ` (source: ${originalLine}; binding: 'title')`;
const wrappedContext = ` (source: ${wrappedLine})`;
const opaqueReceivers = [
  "Property 'missing' does not exist on type 'SYNTHETIC_OPAQUE_A<{\n  alpha: string;\n}>'.",
  "SYNTHETIC_OPAQUE_B\n  receiver printer is deliberately unowned\n  (source: embedded opaque text)",
] as const;

function mockDto(id: Case, receiver: string = opaqueReceivers[0]) {
  const files = sources(input("3.4.23"), "before", id);
  const paths = files
    .filter((file) => file.path.startsWith("src/"))
    .map((file) => file.path)
    .sort();
  const facts = id === "Repair" ? [] : [...originalFacts];
  const rows = paths.map((file) => ({
    file,
    diagnostics: facts
      .filter((fact) => fact.file === file)
      .map((fact) => {
        const suffix = fact.column === 20 ? titleContext : context;
        return `error:${fact.line}:${fact.column} [${fact.code}] ${fact.message ?? receiver}${suffix}`;
      }),
  }));
  if (id === "WrappedSupplement")
    rows
      .find((row) => row.file === "src/Wrapped.vue")!
      .diagnostics.push(
        "error:2:34 [TS2339] Property 'toFixed' does not exist on type 'boolean'." + wrappedContext,
      );
  return {
    files,
    dto: {
      files: rows,
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions: JSON.parse(files[1]!.source).compilerOptions,
          files: paths,
        },
      ],
      errorCount: facts.length + (id === "WrappedSupplement" ? 1 : 0),
      warningCount: 0,
      fileCount: paths.length,
    },
  };
}
type Mock = ReturnType<typeof mockDto>;
const refuses = (action: () => void) => assert.throws(action, assert.AssertionError);
function withPacket(action: (packet: ReturnType<typeof retain>, root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11339-renderer-")));
  try {
    const f = mockDto("Original");
    const packet = retain(recorder(root), "mock", ["vize", "check", "-f", "json"], root, "start", {
      pid: 123,
      status: 1,
      signal: null,
      stdout: Buffer.from(JSON.stringify(f.dto)),
      stderr: Buffer.from("exact unowned progress\r\n"),
    });
    action(packet, root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function diagnostic(f: Mock, index: number) {
  return f.dto.files.find((row) => row.file === "src/Invalid.vue")!.diagnostics[index]!;
}
function mutateDiagnostic(f: Mock, index: number, change: (value: string) => string) {
  const rows = f.dto.files.find((row) => row.file === "src/Invalid.vue")!.diagnostics;
  rows[index] = change(rows[index]!);
}

test("the four reported bases and separately authored Wrapped fact include their exact source renderer", () =>
  withPacket((packet) => {
    for (const id of ["Original", "RestoreOriginal", "WrappedSupplement"] as const) {
      const f = mockDto(id);
      const result = assessFacts(
        "after",
        id,
        { ...packet, stdout: JSON.stringify(f.dto) },
        f.files,
      );
      assert.equal(result.rows.length, id === "WrappedSupplement" ? 6 : 5);
      assert.equal(result.publicAcceptance, false);
      assert.deepEqual(
        result.rows.slice(0, 4).map((row) => row.message),
        [
          originalFacts[0].message + titleContext,
          originalFacts[1].message + context,
          originalFacts[2].message + context,
          originalFacts[3].message + context,
        ],
      );
      if (id === "WrappedSupplement")
        assert.equal(
          result.rows[5]!.message,
          "Property 'toFixed' does not exist on type 'boolean'." + wrappedContext,
        );
    }
  }));

test("two opaque multiline fifth printers are retained whole while only the input-owned finding is assessed", () =>
  withPacket((packet) => {
    for (const receiver of opaqueReceivers) {
      const f = mockDto("Original", receiver);
      const result = assessFacts(
        "after",
        "Original",
        { ...packet, stdout: JSON.stringify(f.dto) },
        f.files,
      );
      assert.equal(result.rows[4]!.message, receiver + context);
      assert.equal(result.rows[4]!.raw, diagnostic(f, 4));
      assert.deepEqual(result.dto, f.dto);
      assert.equal(result.missingReceiverWholeMessageOwned, false);
      assert.equal(result.diagnosticEndOrLengthClaimed, false);
      assert.equal(result.diagnosticPrinterTextClaimedForFifth, false);
      assert.deepEqual(result.missingMemberIntent, {
        findingObserved: true,
        file: "src/Invalid.vue",
        line1: 2,
        startColumn1: 107,
        code: "TS2339",
        authoredToken: "missing",
        authoredUtf16Interval0: [106, 113],
        outputEnd: null,
        outputLength: null,
        receiverTextClaimed: false,
      });
      assert.equal(result.publicAcceptance, false);
    }
  }));

test("source context, title binding and each owned base cannot be omitted or fitted", () =>
  withPacket((packet) => {
    const mutations: Array<(f: Mock) => void> = [
      (f) => mutateDiagnostic(f, 0, (s) => s.replace(titleContext, context)),
      (f) => mutateDiagnostic(f, 1, (s) => s.replace(context, titleContext)),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace(context, "")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("{{ missing }}", "{{ other }}")),
      (f) => mutateDiagnostic(f, 4, (s) => s + " extra"),
      ...[0, 1, 2, 3].map(
        (index) => (f: Mock) =>
          mutateDiagnostic(f, index, (s) =>
            s.replace("does not exist", "MUTATED").replace("Argument", "MUTATED"),
          ),
      ),
    ];
    for (const mutate of mutations) {
      const f = mockDto("Original");
      mutate(f);
      refuses(() =>
        assessFacts("after", "Original", { ...packet, stdout: JSON.stringify(f.dto) }, f.files),
      );
    }
    const wrapped = mockDto("WrappedSupplement");
    wrapped.dto.files.find((row) => row.file === "src/Wrapped.vue")!.diagnostics[0] =
      "error:2:34 [TS2339] Property 'toFixed' does not exist on type 'boolean'." + context;
    refuses(() =>
      assessFacts(
        "after",
        "WrappedSupplement",
        { ...packet, stdout: JSON.stringify(wrapped.dto) },
        wrapped.files,
      ),
    );
  }));

test("file, code, exact canonical start header, ordering and population remain closed", () =>
  withPacket((packet) => {
    const mutations: Array<(f: Mock) => void> = [
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("error:2:107", "error:2:108")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("error:2:107", "error:3:107")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("[TS2339]", "[TS2345]")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("error:2:107", "error:02:107")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace("error:", "warning:")),
      (f) => mutateDiagnostic(f, 4, (s) => s.replace(" [TS2339]", " TS2339")),
      (f) => {
        f.dto.files.find((row) => row.file === "src/Invalid.vue")!.file = "src/Else.vue";
      },
      (f) => {
        f.dto.files.find((row) => row.file === "src/Invalid.vue")!.diagnostics.reverse();
      },
      (f) => {
        f.dto.files.find((row) => row.file === "src/Invalid.vue")!.diagnostics[4] = diagnostic(
          f,
          3,
        );
      },
      (f) => {
        f.dto.files.find((row) => row.file === "src/Invalid.vue")!.diagnostics.pop();
        f.dto.errorCount--;
      },
      (f) => {
        f.dto.files
          .find((row) => row.file === "src/Invalid.vue")!
          .diagnostics.push(diagnostic(f, 4));
        f.dto.errorCount++;
      },
    ];
    for (const mutate of mutations) {
      const f = mockDto("Original");
      mutate(f);
      refuses(() =>
        assessFacts("after", "Original", { ...packet, stdout: JSON.stringify(f.dto) }, f.files),
      );
    }
  }));

test("complete DTO, program, counters and frozen input joins retain strict refusals", () =>
  withPacket((packet) => {
    const mutations: Array<(f: Mock) => void> = [
      (f) => {
        f.dto.files.pop();
      },
      (f) => {
        f.dto.fileCount = 0;
      },
      (f) => {
        f.dto.errorCount = 0;
      },
      (f) => {
        f.dto.warningCount = 1;
      },
      (f) => {
        f.dto.programs[0]!.files.reverse();
      },
      (f) => {
        f.dto.programs[0]!.compilerOptions.strict = false;
      },
      (f) => {
        f.files.find((row) => row.path === "src/Invalid.vue")!.source += "\n";
      },
    ];
    for (const mutate of mutations) {
      const f = mockDto("Original");
      mutate(f);
      refuses(() =>
        assessFacts("after", "Original", { ...packet, stdout: JSON.stringify(f.dto) }, f.files),
      );
    }
    const f = mockDto("Original");
    refuses(() =>
      assessFacts(
        "after",
        "Original",
        { ...packet, stdout: JSON.stringify({ ...f.dto, extra: true }) },
        f.files,
      ),
    );
    refuses(() => assessFacts("after", "Original", { ...packet, status: 0 }, f.files));
    refuses(() => assessFacts("after", "Original", { ...packet, signal: "SIGTERM" }, f.files));
  }));

test("BEFORE remains unassessed, repaired zero remains all-false, and whole stderr stays unchanged", () =>
  withPacket((packet, root) => {
    const f = mockDto("Original");
    const before = assessFacts("before", "Original", packet, f.files);
    assert.equal(before.assessment, "PUBLIC_BEFORE_RETAINED_UNASSESSED");
    assert.equal(before.publicAcceptance, false);
    assert.equal(before.missingMemberIntent, undefined);
    assert.deepEqual(
      readFileSync(join(root, "mock.stderr.raw")),
      Buffer.from("exact unowned progress\r\n"),
    );
    const repair = mockDto("Repair");
    const result = assessFacts(
      "after",
      "Repair",
      { ...packet, status: 0, stdout: JSON.stringify(repair.dto) },
      repair.files,
    );
    assert.equal(result.publicAcceptance, false);
    assert.equal(result.missingMemberIntent, null);
    assert.equal(result.diagnosticEndOrLengthClaimed, false);
  }));
