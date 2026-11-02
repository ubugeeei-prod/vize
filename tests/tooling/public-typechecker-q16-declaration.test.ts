import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { profileFor } from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  declarationData,
  declarationInputs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/declaration-data.ts";
import {
  assessDeclarationConsumer,
  consumerInputImages,
  retainAndCopyDeclarations,
  verifyDeclarationCopies,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/declaration.ts";
import {
  capture,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";

function temporary(run: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-declaration-")));
  try {
    run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}
function setup(root: string) {
  const output = join(root, "out"),
    consumer = join(root, "consumer"),
    evidence = join(root, "evidence");
  for (const path of [output, consumer, evidence, join(consumer, "src")]) mkdirSync(path);
  const bytes = [
    Buffer.from("export default class Foo {}\n"),
    Buffer.from("export type Helper = unknown;\n"),
  ];
  declarationData.outputFiles.forEach((name, i) => writeFileSync(join(output, name), bytes[i]!));
  const inputs = declarationInputs(profileFor(), false);
  for (const file of inputs) writeFileSync(join(consumer, file.path), file.source);
  return { output, consumer, evidence, bytes, inputs };
}
const processOutput = (status: number, stdout: string, stderr: string): ProcessOutput => ({
  status,
  stdout: Buffer.from(stdout),
  stderr: Buffer.from(stderr),
  signal: null,
  pid: 42,
  error: null,
  capacityLimited: false,
});

test("A64 binds noncausal known-good source and exact original commands/four intended states", () => {
  const profile = profileFor();
  assert.equal(profile.subset, "A64");
  assert.equal(declarationData.knownGoodSource, "07d1feb1bb19d862b30aea10fe3465a08a8b1239");
  assert.deepEqual(declarationData.producerArgs, [
    "npx",
    "vize",
    "check",
    "--declaration",
    "--declaration-dir",
    "../out",
    "src/Foo.vue",
  ]);
  assert.deepEqual(declarationData.consumerArgs, [
    "npx",
    "vue-tsc",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
  assert.deepEqual(
    declarationData.states.map((row) => row.expected.status),
    [0, 2, 2, 0],
  );
  assert.equal(declarationData.states[0]!.source, declarationData.states[3]!.source);
  for (const producer of [true, false])
    for (const file of declarationInputs(profile, producer))
      assert.equal(
        file.source,
        file.path === "package.json"
          ? declarationData.packageJson.replace('"vize": "0.638.0"', '"vize": "0.643.0"')
          : file.path === "tsconfig.json"
            ? declarationData.config
            : producer
              ? declarationData.producer
              : declarationData.consumer,
      );
  assert(!declarationData.producer.includes("onClick"));
});
test("only actual whole generated two-file bytes are retained and copied; mutation refuses", () =>
  temporary((root) => {
    const x = setup(root),
      record = recorder(x.evidence),
      files = retainAndCopyDeclarations(x.output, x.consumer, record);
    verifyDeclarationCopies(x.output, x.consumer, files);
    consumerInputImages(x.consumer, x.inputs);
    files.forEach((file, i) => {
      assert.deepEqual(file.bytes, x.bytes[i]);
      assert.deepEqual(readFileSync(join(x.evidence, "emitted-" + file.name + ".raw")), x.bytes[i]);
      assert.deepEqual(readFileSync(join(x.evidence, "copied-" + file.name + ".raw")), x.bytes[i]);
      assert.deepEqual(readFileSync(join(x.consumer, "src/vendor", file.name)), x.bytes[i]);
    });
    writeFileSync(join(x.consumer, "src/vendor", files[0]!.name), "changed");
    assert.throws(
      () => verifyDeclarationCopies(x.output, x.consumer, files),
      assert.AssertionError,
    );
    assert(
      readFileSync(join(x.evidence, "whole-declaration-transfer.json"), "utf8").includes(
        '"sourceDeclarationsInjected": false',
      ),
    );
  }));
test("extra or symlinked output refuses before copy/consumer callback", () => {
  for (const mode of ["extra", "link"]) {
    temporary((root) => {
      const x = setup(root);
      if (mode === "extra") writeFileSync(join(x.output, "unexpected.d.ts"), "extra");
      else {
        const path = join(x.output, declarationData.outputFiles[0]!);
        rmSync(path);
        const held = join(root, "held.d.ts");
        writeFileSync(held, x.bytes[0]!);
        symlinkSync(held, path);
      }
      let executed = 0;
      const row = capture(
        recorder(x.evidence),
        "refused",
        ["inert"],
        x.consumer,
        () => {
          retainAndCopyDeclarations(x.output, x.consumer, recorder(x.evidence));
        },
        () => {
          executed++;
          return processOutput(0, "", "");
        },
      );
      assert.equal(executed, 0);
      assert.equal(row.executed, false);
      assert(!existsSync(join(x.consumer, "src/vendor")));
    });
  }
});
test("consumer same-byte link and private mutation refuse before callback without value egress", () => {
  for (const mode of ["link", "private"]) {
    temporary((root) => {
      const x = setup(root),
        files = retainAndCopyDeclarations(x.output, x.consumer, recorder(x.evidence)),
        path = join(x.consumer, "src/Probe.vue"),
        secret = "PRIVATE_DECLARATION_CANARY";
      if (mode === "link") {
        const held = join(root, "same.vue");
        writeFileSync(held, declarationData.consumer);
        rmSync(path);
        symlinkSync(held, path);
      } else writeFileSync(path, secret);
      let executed = 0;
      const row = capture(
        recorder(x.evidence),
        "input-refused",
        ["inert"],
        x.consumer,
        () => {
          consumerInputImages(x.consumer, x.inputs);
          verifyDeclarationCopies(x.output, x.consumer, files);
        },
        () => {
          executed++;
          return processOutput(0, "", "");
        },
      );
      assert.equal(executed, 0);
      assert.equal(row.executed, false);
      assert(
        !readFileSync(join(x.evidence, "input-refused.process.json"), "utf8").includes(secret),
      );
    });
  }
});
test("all four whole consumer vectors require exact status/streams and retain raw before assessment", () =>
  temporary((root) => {
    const record = recorder(root);
    for (const [i, state] of declarationData.states.entries()) {
      const row = capture(
        record,
        "state-" + i,
        ["inert"],
        root,
        () => {},
        () => processOutput(state.expected.status, state.expected.stdout, state.expected.stderr),
      );
      assert.deepEqual(readFileSync(row.stdoutImage.path), Buffer.from(state.expected.stdout));
      assert.deepEqual(readFileSync(row.stderrImage.path), Buffer.from(state.expected.stderr));
      assessDeclarationConsumer(state, row);
      assert.equal(row.qualification, false);
      for (const mutation of [
        { status: state.expected.status === 0 ? 2 : 0 },
        { stdout: row.stdout + "extra" },
        { stderr: "extra" },
        { signal: "SIGTERM" },
        { wholeStreamsRetained: false },
        { processError: "UNKNOWN" },
      ])
        assert.throws(
          () => assessDeclarationConsumer(state, { ...row, ...mutation }),
          assert.AssertionError,
        );
    }
  }));
