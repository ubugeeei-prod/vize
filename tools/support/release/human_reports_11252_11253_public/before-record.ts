import assert from "node:assert/strict";
import type { Bundle, Request, ReleaseAuthority } from "./model.ts";
import { recipes, beforeCommands } from "./recipes.ts";
import { argumentsFor } from "./commands.ts";
import { manifestBytes } from "./installation.ts";
import { validateRecordedInstall, sameRelease } from "./installation-record.ts";
import { validateCapture, compareOutcome } from "./processes.ts";
import { sha256 } from "./value.ts";
export function validateBeforeRecord(
  bundle: Bundle,
  request: Request,
  authority: ReleaseAuthority,
): void {
  assert.equal(bundle.request.role, request.role);
  assert.equal(bundle.request.mode, "CAPTURE_BEFORE");
  assert.equal(bundle.request.release.phase, "before");
  assert.equal(bundle.request.release.issue, request.fixture.issue);
  assert.deepEqual(bundle.request.release.fixes, []);
  assert.deepEqual(bundle.request.fixture, request.fixture);
  sameRelease(bundle.release, authority);
  const recipe = recipes[request.role],
    ids = beforeCommands[request.role];
  assert.deepEqual(
    bundle.request.commands.map((v) => v.id),
    ids,
  );
  assert.deepEqual(
    bundle.observations.map((v) => v.id),
    ids,
  );
  assert.deepEqual(
    bundle.installs.map((v) => v.kind),
    recipe.tsrs ? ["original", "tsrs-supplement"] : ["original"],
  );
  for (const install of bundle.installs)
    validateRecordedInstall(install, request.fixture, recipe, authority);
  const observer = bundle.controller.files.filter((f) =>
    f.path.endsWith("/tools/support/release/human_reports_11252_11253_public/native-observer.ts"),
  );
  assert.equal(observer.length, 1);
  const companionIds: string[] = [];
  for (const plan of bundle.request.commands) {
    const supplement = plan.id.includes("tsrs"),
      kind = supplement ? "tsrs-supplement" : "original",
      install = bundle.installs.find((v) => v.kind === kind);
    assert(install);
    assert.equal(plan.consumer, kind);
    assert.equal(plan.executable, install.pnpm.path);
    assert.deepEqual(plan.argv, argumentsFor(bundle.request, plan.id));
    const expected = request.fixture.files.map((f) => {
      if (f.path !== "package.json") return f;
      const raw = manifestBytes(request.fixture, authority.version, supplement);
      return { path: f.path, base64: raw.toString("base64"), sha256: sha256(raw) };
    });
    assert.deepEqual(plan.sourceFiles, expected);
    assert.equal(plan.oracle, null);
    assert.equal(plan.requiresNativeLoad, plan.id !== "original-vue-tsc");
    const original =
      plan.id === "original-pnpm-check" ||
      (request.role === "imports-vue-ts-11253" && plan.id !== "original-cli-json");
    assert.equal(plan.purpose, original ? "original" : "supplement");
    assert.equal(
      plan.adaptation,
      original
        ? null
        : "Explicit report-note/supplement command; original script, sources and compiler settings remain separate",
    );
    const row = bundle.observations.find((v) => v.id === plan.id);
    assert(row);
    validateCapture(plan, row, install);
    if (plan.requiresNativeLoad) {
      companionIds.push(plan.id);
      const companion = bundle.companions?.find((v) => v.id === plan.id);
      assert(companion);
      validateCapture(plan, companion, install, true, observer[0]!.path);
    }
  }
  assert.deepEqual(
    bundle.companions?.map((v) => v.id),
    companionIds,
    "closed companion vector required",
  );
  const expected = request.publicBefore;
  assert(expected);
  assert.deepEqual(
    expected.oracles.map((v) => v.id),
    ids,
  );
  for (const oracle of expected.oracles) {
    assert.deepEqual(Object.keys(oracle).sort(), ["id", "status", "stderr", "stdout"]);
    const row = bundle.observations.find((v) => v.id === oracle.id);
    assert(row);
    compareOutcome(oracle, row);
  }
}
