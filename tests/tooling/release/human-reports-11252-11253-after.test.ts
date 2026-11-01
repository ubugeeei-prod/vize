import assert from "node:assert/strict";
import test from "node:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
  existsSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import {
  afterPlan,
  requiredFixPaths,
  validateAfterInputs,
  inputPlanGitBlob,
  inputPlanSha256,
  instantiatedInputSha256,
  unqualifiedAfterVerdict,
} from "../../../tools/support/release/human_reports_11252_11253_public/after-plan.ts";
import {
  assertInputState,
  checkCaseManagers,
  inputStateWitness,
  restoreInputState,
} from "../../../tools/support/release/human_reports_11252_11253_public/input-state.ts";
import { authenticateAfterInputs } from "../../../tools/support/release/human_reports_11252_11253_public/after-admission.ts";
import { managerFiles } from "../../../tools/support/release/human_reports_11252_11253_public/optional-residency.ts";
import { loadOriginalFixture } from "../../../tools/support/release/human_reports_11252_11253_public/fixtures.ts";
import {
  requireCaptureReady,
  requireComplete,
} from "../../../tools/support/release/human_reports_11252_11253_public/recipes.ts";
import {
  bytes,
  gitBlob,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import type { InputState } from "../../../tools/support/release/human_reports_11252_11253_public/after-plan.ts";
import type {
  Request,
  Role,
  ReleaseAuthority,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";

const version = "9.0.0"; // Synthetic target only. No published version is selected by this test.
const roles: Role[] = ["runtime-prop-null-11252", "imports-vue-ts-11253"];
const root = process.env.HUMAN_AFTER_TEST_ROOT ?? tmpdir();
mkdirSync(root, { recursive: true });
function fixture<T>(body: (path: string) => T): T {
  const path = mkdtempSync(join(root, "human-after-pure-"));
  try {
    return body(path);
  } finally {
    rmSync(path, { recursive: true, force: true });
  }
}
function seed(root: string, state: InputState): void {
  for (const f of state.present) {
    const path = join(root, f.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, bytes(f.base64));
  }
}
function request(role: Role): Request {
  const issue = role === roles[0] ? 11252 : 11253;
  return {
    schema: "vize.humanReports11252_11253.request.v1",
    role,
    mode: "UNQUALIFIED_AFTER",
    reviewedController: {
      head: "a".repeat(40),
      files: [{ path: "mock", bytes: 1, sha256: "b".repeat(64), gitBlob: "c".repeat(40) }],
    },
    fixture: loadOriginalFixture(role),
    release: {
      issue,
      phase: "after",
      version,
      releaseRun: "123",
      capsuleBlob: "d".repeat(40),
      fixes: [
        {
          issue,
          pull: issue === 11252 ? 11257 : 11258,
          originalHead: "e".repeat(40),
          merge:
            issue === 11252
              ? "170288d4401511f874d958d709d28f5e001a2e1c"
              : "64996d903726427ba8c8ea5ac99bfb1042f77a6f",
          paths: [...requiredFixPaths[issue]],
        },
      ],
    },
    commands: afterPlan(role, version).map((p) => p.command),
    sourceAdmission: {
      commit: "e".repeat(40),
      run: "456",
      receiptBlob: "f".repeat(40),
      receiptSha256: "0".repeat(64),
    },
    oracleOwnerReceipt: null,
    publicBefore: null,
    capturedBefore: {
      kind: "CAPTURE_ONLY",
      controllerRun: "789",
      controllerHead: "a".repeat(40),
      artifactId: 1,
      archiveDigest: "sha256:" + "b".repeat(64),
      captureMember: "unqualified-capture.json",
      captureMemberSha256: "c".repeat(64),
      manualRequestBlob: "d".repeat(40),
    },
  };
}
test("source-bound plan has exactly23 originals and22 companions with no output law", () => {
  const plans = roles.flatMap((r) => afterPlan(r, version));
  assert.equal(plans.length, 23);
  assert.equal(plans.filter((p) => p.command.requiresNativeLoad).length, 22);
  assert(plans.every((p) => p.command.oracle === null));
  for (const role of roles) {
    validateAfterInputs(request(role));
    requireCaptureReady(request(role));
  }
});
test("unknown target/version and unregistered role cannot instantiate inputs", () => {
  for (const bad of ["", "UNKNOWN", "0.627.0", "v1.2.3"])
    assert.throws(() => afterPlan(roles[0]!, bad));
  assert.throws(() => afterPlan("other" as Role, version));
});
test("finite fix scope includes every production path and original fixture", () => {
  assert.equal(requiredFixPaths[11252].length, 9);
  assert.equal(requiredFixPaths[11253].length, 2);
  assert(
    requiredFixPaths[11252].includes(
      "crates/vize_canon/src/batch/virtual_project/setup_props/runtime_props/shape.rs",
    ),
  );
  for (const role of roles)
    for (const path of requiredFixPaths[request(role).release.issue]) {
      const r = request(role);
      r.release.fixes[0]!.paths = r.release.fixes[0]!.paths.filter((p) => p !== path);
      assert.throws(() => validateAfterInputs(r), /missing required fix\/fixture path/);
    }
  const r = request(roles[0]!);
  r.release.fixes[0]!.merge = "a".repeat(40);
  assert.throws(() => validateAfterInputs(r));
});
test("missing source admission or genuine BEFORE cannot become capture-ready", () => {
  const r = request(roles[0]!);
  r.sourceAdmission = null;
  assert.throws(() => requireCaptureReady(r));
  const s = request(roles[0]!);
  s.capturedBefore = null;
  assert.throws(() => requireCaptureReady(s));
});
test("strict acceptance still rejects unqualified mode and absent whole oracles", () => {
  assert.throws(() => requireComplete(request(roles[0]!)));
  const r = request(roles[0]!);
  r.mode = "ACCEPT_AFTER";
  assert.throws(() => requireComplete(r), /oracle receipt/);
});
test("reordered/omitted/extra commands, changed argv and source bytes refuse", () => {
  for (const mutate of [
    (r: Request) => {
      r.commands.reverse();
    },
    (r: Request) => {
      r.commands.pop();
    },
    (r: Request) => {
      r.commands.push(structuredClone(r.commands[0]!));
    },
    (r: Request) => {
      r.commands[0]!.argv.push("--unexpected");
    },
    (r: Request) => {
      r.commands[0]!.sourceFiles[0]!.base64 = Buffer.from("tamper").toString("base64");
    },
    (r: Request) => {
      r.commands[0]!.adaptation = "invented";
    },
  ]) {
    const r = request(roles[0]!);
    mutate(r);
    assert.throws(() => validateAfterInputs(r));
  }
});
test("a source JSON contract or observed output cannot be inserted as capture oracle", () => {
  const r = request(roles[1]!);
  r.commands[1]!.oracle = {
    id: r.commands[1]!.id,
    status: 0,
    stdout: { kind: "json", value: {} },
    stderr: { kind: "bytes", base64: "" },
    sourceOwnedBy: { commit: "a".repeat(40), path: "mock", sha256: "b".repeat(64) },
  };
  assert.throws(() => validateAfterInputs(r));
});
test("all45 registered original/companion transitions restore exact bytes and absences", () => {
  let transitions = 0,
    typedRemovedBeforeOriginalCompanion = false;
  for (const role of roles)
    fixture((base) => {
      const plans = afterPlan(role, version),
        consumers = [...new Set(plans.map((p) => p.command.consumer))];
      for (const consumer of consumers) {
        const initial = plans.find((p) => p.command.consumer === consumer)!;
        const path = join(base, consumer);
        mkdirSync(path);
        seed(path, initial.before);
      }
      for (const companion of [false, true])
        for (const plan of plans) {
          if (companion && !plan.command.requiresNativeLoad) continue;
          const path = join(base, plan.command.consumer),
            images = plans
              .filter((p) => p.command.consumer === plan.command.consumer)
              .flatMap((p) => [p.before, p.after]);
          restoreInputState(path, plan.before, images, () => {});
          assertInputState(path, plan.after);
          if (role === roles[0] && companion && plan.command.id === "original-pnpm-check") {
            assert(!existsSync(join(path, "src/Typed.vue")));
            typedRemovedBeforeOriginalCompanion = true;
          }
          transitions++;
        }
    });
  assert.equal(transitions, 45);
  assert(typedRemovedBeforeOriginalCompanion);
});
test("unknown Typed.vue bytes are preserved and refuse before removal", () =>
  fixture((root) => {
    const p = afterPlan(roles[0]!, version),
      original = p[0]!.before;
    seed(root, original);
    writeFileSync(join(root, "src/Typed.vue"), "unexpected authored source");
    assert.throws(() =>
      restoreInputState(
        root,
        original,
        p.flatMap((x) => [x.before, x.after]),
        () => {},
      ),
    );
    assert.equal(readFileSync(join(root, "src/Typed.vue"), "utf8"), "unexpected authored source");
  }));
test("unexpected source files and links are never cleaned or followed", () =>
  fixture((root) => {
    const p = afterPlan(roles[0]!, version),
      original = p[0]!.before;
    seed(root, original);
    writeFileSync(join(root, "src/Unowned.ts"), "preserve me");
    assert.throws(() =>
      restoreInputState(
        root,
        original,
        p.map((x) => x.before),
        () => {},
      ),
    );
    assert.equal(readFileSync(join(root, "src/Unowned.ts"), "utf8"), "preserve me");
    rmSync(join(root, "src/Unowned.ts"));
    symlinkSync(join(root, "src/Box.vue"), join(root, "src/Unowned.ts"));
    assert.throws(() =>
      restoreInputState(
        root,
        original,
        p.map((x) => x.before),
        () => {},
      ),
    );
    assert(existsSync(join(root, "src/Unowned.ts")));
  }));
test("a mixture of registered HashVue and Original images still refuses", () =>
  fixture((root) => {
    const p = afterPlan(roles[1]!, version),
      original = p[0]!.before,
      hash = p.find((x) => x.command.id === "hash-vue")!.before;
    seed(root, original);
    writeFileSync(
      join(root, "package.json"),
      bytes(hash.present.find((f) => f.path === "package.json")!.base64),
    );
    assert.throws(() =>
      restoreInputState(
        root,
        original,
        p.map((x) => x.before),
        () => {},
      ),
    );
  }));
test("an unregistered empty source directory is preserved and refuses", () =>
  fixture((root) => {
    const p = afterPlan(roles[0]!, version);
    seed(root, p[0]!.before);
    mkdirSync(join(root, "src/UnownedDirectory"));
    assert.throws(
      () =>
        restoreInputState(
          root,
          p[0]!.before,
          p.map((x) => x.before),
          () => {},
        ),
      /unexpected source directory/,
    );
    assert(existsSync(join(root, "src/UnownedDirectory")));
  }));
test("case package binding preserves initial lock/workspace/npm bytes and BEFORE equality", () =>
  fixture((root) => {
    const p = afterPlan(roles[1]!, version),
      original = p[0]!.before,
      hash = p.find((x) => x.command.id === "hash-vue")!.before;
    seed(root, original);
    writeFileSync(join(root, "pnpm-lock.yaml"), "synthetic lock\n");
    writeFileSync(join(root, "pnpm-workspace.yaml"), "synthetic workspace\n");
    writeFileSync(join(root, ".npmrc"), "synthetic config\n");
    const initial = managerFiles(root);
    restoreInputState(
      root,
      hash,
      p.map((x) => x.before),
      () => {},
    );
    const manifest = bytes(hash.present.find((f) => f.path === "package.json")!.base64);
    checkCaseManagers(initial, managerFiles(root), manifest);
    assert.throws(() => checkCaseManagers(initial, managerFiles(root)));
    for (const name of ["pnpm-lock.yaml", "pnpm-workspace.yaml", ".npmrc", "package.json"]) {
      const path = join(root, name),
        old = readFileSync(path);
      writeFileSync(path, Buffer.concat([old, Buffer.from("tamper")]));
      assert.throws(() => checkCaseManagers(initial, managerFiles(root), manifest));
      writeFileSync(path, old);
    }
  }));
test("pre-transition manager refusal happens before source restoration", () =>
  fixture((root) => {
    const p = afterPlan(roles[0]!, version);
    seed(root, p[0]!.before);
    assert.throws(() =>
      restoreInputState(
        root,
        p[3]!.before,
        p.map((x) => x.before),
        () => {},
        () => {
          throw new Error("mocked manager mutation");
        },
      ),
    );
    assert(!existsSync(join(root, "src/Typed.vue")));
    const witness = inputStateWitness(root, p[0]!.before) as {
      files: Array<{ path: string; state: string }>;
    };
    assert.equal(witness.files.find((f) => f.path === "src/Typed.vue")!.state, "ABSENT");
  }));
test("all capture qualification flags remain literalfalse independently of process outcomes", () => {
  const v = unqualifiedAfterVerdict(roles[0]!) as Record<string, unknown>;
  for (const k of [
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
    "outcomeAssessed",
    "knownBugReproductionJudged",
  ])
    assert.equal(v[k], false);
  assert.equal(v.backend, "UNKNOWN");
  assert.equal(v.mode, "UNQUALIFIED_AFTER");
});
test("input admission rejects self-output and wrong target/plan before run API reads", async () => {
  for (const mutation of ["oracles", "target", "plan"] as const) {
    const r = request(roles[0]!),
      admission = r.sourceAdmission!;
    const receipt: Record<string, unknown> = {
      schema: "vize.humanReportPublic.sourceInputAdmission.v1",
      mode: "UNQUALIFIED_AFTER",
      role: r.role,
      fixtureReceiptSha256: r.fixture.receiptSha256,
      commit: admission.commit,
      run: admission.run,
      scope: "source-adapted-not-public-target-execution",
      publicTargetExecuted: false,
      outcomeAssessed: false,
      inputPlanSha256,
      inputPlanGitBlob,
      instantiatedInputsSha256: instantiatedInputSha256(r.role, version),
      release: {
        version,
        releaseRun: "123",
        workflowHead: "a".repeat(40),
        source: "b".repeat(40),
        candidate: "c".repeat(40),
        capsuleBlob: r.release.capsuleBlob,
      },
      fix: r.release.fixes[0],
    };
    if (mutation === "oracles") receipt.oracles = [];
    if (mutation === "target")
      (receipt.release as Record<string, unknown>).candidate = "d".repeat(40);
    if (mutation === "plan") receipt.inputPlanSha256 = "0".repeat(64);
    const raw = Buffer.from(JSON.stringify(receipt));
    admission.receiptBlob = gitBlob(raw);
    admission.receiptSha256 = sha256(raw);
    let reads = 0;
    const authority = {
      version,
      run: "123",
      workflowHead: "a".repeat(40),
      source: "b".repeat(40),
      candidate: "c".repeat(40),
    } as ReleaseAuthority;
    await assert.rejects(() =>
      authenticateAfterInputs(r, authority, async (path) => {
        reads++;
        assert.equal(path, `git/blobs/${admission.receiptBlob}`);
        return { encoding: "base64", content: raw.toString("base64") };
      }),
    );
    assert.equal(reads, 1);
  }
});
