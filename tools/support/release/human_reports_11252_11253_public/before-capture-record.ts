import assert from "node:assert/strict";
import type {
  CapturedBeforeReference,
  Request,
  ReleaseAuthority,
  UnqualifiedCapture,
} from "./model.ts";
import { beforeCommands, recipes } from "./recipes.ts";
import { argumentsFor } from "./commands.ts";
import { manifestBytes } from "./installation.ts";
import { sameRelease } from "./installation-record.ts";
import { validateCapture } from "./processes.ts";
import { validateNativeLoadRecord } from "./native-loader-prerequisite.ts";
import { validateBinConfiguration } from "./bin-configuration.ts";
import { assertRecordedBinPolicy } from "./dependency-bin-policy.ts";
import { array, bytes, object, sha256, text, wholeJson } from "./value.ts";

export interface CapturePolicyEntry {
  issue: 11252 | 11253;
  role: Request["role"];
  fixtureReceiptSha256: string;
  ownerFixMerge: string;
  reference: CapturedBeforeReference;
  artifact: {
    compressedBytes: number;
    entries: number;
    decodedBytes: number;
    jobId: number;
    sourceFileCount: number;
    knownWorkflowPath: string;
    runAttempt: number;
  };
  judgment: Record<string, unknown> | null;
}
export function requireCaptureJudgment(entry: CapturePolicyEntry): Record<string, unknown> {
  const j = entry.judgment;
  assert(j, "selected role lacks an independent before judgment");
  assert.deepEqual(
    Object.keys(j).sort(),
    [
      "schema",
      "role",
      "issue",
      "fixtureReceiptSha256",
      "ownerFixMerge",
      "reference",
      "originalFields",
      "completeOriginalAndCompanionPacketsReviewed",
      "knownBugReproduction",
      "postQualificationRefused",
      "scope",
      "sourceOwnedTargetOutputsClaimed",
      "afterOutputsSupplied",
      "sourceAdaptedOutputsTransferred",
      "accepted",
      "productQualified",
      "afterQualified",
      "issueQualified",
      "reviewer",
      "notes",
    ].sort(),
  );
  assert.equal(j.schema, "vize.humanReportPublic.beforeCaptureJudgment.v1");
  for (const key of ["role", "issue", "fixtureReceiptSha256", "ownerFixMerge"] as const)
    assert.equal(j[key], entry[key]);
  assert.deepEqual(j.reference, entry.reference);
  assert.equal(j.completeOriginalAndCompanionPacketsReviewed, true);
  assert.equal(j.knownBugReproduction, "OBSERVED");
  assert.equal(j.postQualificationRefused, true);
  assert.equal(j.scope, "ORIGINAL_PUBLIC_BEFORE_ONLY");
  for (const key of [
    "sourceOwnedTargetOutputsClaimed",
    "afterOutputsSupplied",
    "sourceAdaptedOutputsTransferred",
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
  ])
    assert.equal(j[key], false);
  const owner = entry.issue === 11252 ? "generation_audit" : "gate_audit",
    reviewer = text(j.reviewer);
  // This memo is not an identity credential. The complete independent review
  // is owned by the authenticated controller DATA blob, not this display name.
  assert(reviewer === owner || reviewer.includes("/root/typechecker_lead/" + owner));
  assert(text(j.notes).length > 0);
  return j;
}
/** Recorded paths are compared as strings only. A pending capture cannot enter
 * the strict Bundle/InstallReceipt validators or read an old runner filesystem. */
export function validateBeforeCaptureRecord(
  capture: UnqualifiedCapture,
  manual: Request,
  after: Request,
  release: ReleaseAuthority,
  entry: CapturePolicyEntry,
  members: ReadonlyMap<string, Buffer>,
): void {
  const judgment = requireCaptureJudgment(entry);
  assert.equal(after.mode, "UNQUALIFIED_AFTER");
  assert.equal(after.publicBefore ?? null, null);
  assert.deepEqual(after.capturedBefore, entry.reference);
  assert.equal(capture.schema, "vize.humanReports11252_11253.unqualifiedCapture.v1");
  for (const key of [
    "completeInstalledFileSetQualified",
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
    "outcomeAssessed",
  ] as const)
    assert.equal(capture[key], false);
  assert.equal(manual.mode, "CAPTURE_BEFORE");
  assert.equal(manual.release.phase, "before");
  assert.equal(manual.release.version, "0.627.0");
  assert.equal(manual.release.releaseRun, "41236623057");
  assert.equal(manual.release.capsuleBlob, "0dfa8a25e40e4c045d270171026d46b7471dd4e1");
  assert.deepEqual(manual.release.fixes, []);
  assert.equal(manual.sourceAdmission, null);
  assert.equal(manual.oracleOwnerReceipt, null);
  for (const key of [
    "role",
    "fixture",
    "reviewedController",
    "release",
    "sourceAdmission",
    "oracleOwnerReceipt",
  ] as const)
    assert.deepEqual(capture.request[key], manual[key]);
  assert.equal(manual.role, after.role);
  assert.deepEqual(manual.fixture, after.fixture);
  assert.equal(entry.fixtureReceiptSha256, after.fixture.receiptSha256);
  assert.equal(
    entry.ownerFixMerge,
    after.release.fixes.find((f) => f.issue === entry.issue)?.merge,
  );
  sameRelease(capture.release, release);
  assert.equal(capture.controller.head, entry.reference.controllerHead);
  assert.equal(capture.controller.run, entry.reference.controllerRun);
  assert.equal(capture.controller.job, "public-consumer");
  assert.equal(capture.controller.files.length, entry.artifact.sourceFileCount);
  const sourceRows = manual.reviewedController.files;
  assert.equal(sourceRows.length, entry.artifact.sourceFileCount);
  const workflow = capture.controller.files.filter((f) =>
    f.path.endsWith("/" + entry.artifact.knownWorkflowPath),
  );
  assert.equal(workflow.length, 1);
  const prefix = workflow[0]!.path.slice(0, -entry.artifact.knownWorkflowPath.length);
  assert.equal(
    new Set(capture.controller.files.map((f) => f.path)).size,
    capture.controller.files.length,
  );
  for (const source of sourceRows) {
    const rows = capture.controller.files.filter((f) => f.path === prefix + source.path);
    assert.equal(rows.length, 1);
    assert.equal(rows[0]!.bytes, source.bytes);
    assert.equal(rows[0]!.sha256, source.sha256);
  }
  const recipe = recipes[after.role],
    ids = beforeCommands[after.role];
  assert.deepEqual(
    capture.request.commands.map((c) => c.id),
    ids,
  );
  assert.deepEqual(
    capture.observations.map((r) => r.id),
    ids,
  );
  assert.deepEqual(
    capture.installs.map((i) => i.kind),
    recipe.tsrs ? ["original", "tsrs-supplement"] : ["original"],
  );
  const witnesses = [...members]
    .filter(([name]) => name.endsWith("-public-payload-prerequisites.json"))
    .map(([, body]) => object(wholeJson(body)).value)
    .map(object);
  for (const install of capture.installs) {
    assert.equal(install.schema, "vize.humanReports11252_11253.capturePrerequisites.v1");
    for (const key of [
      "completeInstalledFileSetQualified",
      "accepted",
      "productQualified",
      "afterQualified",
      "issueQualified",
    ] as const)
      assert.equal(install[key], false);
    assert.equal(install.mandatoryExecutableChainsAuthenticated, true);
    assert.equal(install.nodeVersion, recipe.node);
    assert.equal(install.pnpmVersion, recipe.pnpm);
    assert.equal(install.platform, recipe.platform);
    assert.equal(install.arch, recipe.arch);
    const manifest = manifestBytes(
      after.fixture,
      release.version,
      install.kind === "tsrs-supplement",
    );
    assert(bytes(install.packageRawBase64).equals(manifest));
    assert.equal(sha256(bytes(install.lockRawBase64)), install.lockSha256);
    validateBinConfiguration(install.binConfiguration, manifest);
    assertRecordedBinPolicy(install.binPolicy, manual.reviewedController);
    const chain = install.executableChains;
    assert.equal(chain.schema, "vize.humanReports11252_11253.executableChains.v1");
    assert.equal(chain.kind, install.kind);
    assert.deepEqual(chain.recipe, recipe);
    assert.equal(chain.initialManifestBase64, manifest.toString("base64"));
    assert.equal(chain.authenticatedForCaptureOnly, true);
    for (const key of [
      "completeInstalledFileSetQualified",
      "accepted",
      "productQualified",
      "afterQualified",
      "issueQualified",
    ] as const)
      assert.equal(chain[key], false);
    assert.equal(chain.census.root, install.root);
    assert.equal(chain.runtime.root, install.root);
    assert.equal(chain.census.rootPackageSha256, sha256(manifest));
    assert.equal(chain.census.sourceInputPolicyGitBlob, install.binPolicy.gitBlob);
    assert.equal(chain.census.actualWriterRuntimeAttribution, false);
    assert.equal(chain.runtime.managerWriterImageOwner, "UNKNOWN");
    assert.equal(chain.runtime.actualWriterRuntimeAttribution, false);
    assert.deepEqual(chain.runtime.managerEntry, install.pnpm);
    for (const selected of chain.runtime.selected)
      if (selected.command === "node") assert.deepEqual(selected.image, install.node);
    for (const target of chain.census.targets) {
      assert(
        install.packages.some(
          (pkg) =>
            pkg.name === target.ownerName &&
            pkg.version === target.ownerVersion &&
            pkg.directory === target.ownerDirectory &&
            pkg.files.some(
              (file) =>
                file.path === target.publicTarget.path &&
                file.bytes === target.publicTarget.bytes &&
                file.sha256 === target.publicTarget.sha256,
            ),
        ),
      );
      const wrapper = bytes(target.wrapper.rawBase64);
      assert.equal(wrapper.length, target.wrapper.bytes);
      assert.equal(sha256(wrapper), target.wrapper.sha256);
    }
    for (const name of chain.requiredCommands)
      assert.equal(chain.census.targets.filter((t) => t.name === name).length, 1);
    validateNativeLoadRecord(install, release);
    assert.equal(install.sdk.length, 10);
    for (const sdk of release.sdk.filter((f) => f.path !== "package.json")) {
      const image = install.sdk.filter((f) => f.path.endsWith("/" + sdk.path));
      assert.equal(image.length, 1);
      assert.equal(image[0]!.bytes, sdk.bytes);
      assert.equal(image[0]!.sha256, sdk.sha256);
      const copied = members.get(`sdk/${install.kind}/${sdk.path.replaceAll("/", "--")}`);
      assert(copied);
      assert.equal(copied.length, sdk.bytes);
      assert.equal(sha256(copied), sdk.sha256);
    }
    for (const pkg of install.packages) {
      assert.equal(pkg.publicPayloadBytesEqualArchive, true);
      assert.equal(pkg.completeInstalledFileSetQualified, false);
      assert.equal(bytes(pkg.integrity.slice(7)).toString("hex"), pkg.archiveSha512);
      const matching = witnesses.filter(
        (w) =>
          w.package === pkg.name &&
          w.version === pkg.version &&
          w.complete === true &&
          array(w.observations).length > 0 &&
          array(w.observations)
            .map(object)
            .every((o) => text(o.path).startsWith(pkg.directory + "/")),
      );
      assert(matching.length > 0, "recorded public payload witness is required");
      for (const witness of matching) {
        const rows = array(witness.observations).map(object);
        assert.equal(rows.length, pkg.files.length);
        for (const file of pkg.files) {
          const found = rows.filter((r) => r.path === file.path);
          assert.equal(found.length, 1);
          const r = found[0]!;
          assert.equal(r.expectedBytes, file.bytes);
          assert.equal(r.actualBytes, file.bytes);
          assert.equal(r.expectedSha256, file.sha256);
          assert.equal(r.actualSha256, file.sha256);
        }
      }
    }
  }
  const observers = capture.controller.files.filter((f) =>
    f.path.endsWith("/tools/support/release/human_reports_11252_11253_public/native-observer.ts"),
  );
  assert.equal(observers.length, 1);
  const companionIds: string[] = [];
  for (const plan of capture.request.commands) {
    const install = capture.installs.find((i) => i.kind === plan.consumer);
    assert(install);
    assert.equal(plan.executable, install.pnpm.path);
    assert.deepEqual(plan.argv, argumentsFor(after, plan.id));
    assert.equal(plan.oracle, null);
    assert.equal(plan.consumer, plan.id.includes("tsrs") ? "tsrs-supplement" : "original");
    const original =
      plan.id === "original-pnpm-check" ||
      (after.role === "imports-vue-ts-11253" && plan.id !== "original-cli-json");
    assert.equal(plan.purpose, original ? "original" : "supplement");
    assert.equal(
      plan.adaptation,
      original
        ? null
        : "Explicit report-note/supplement command; original script, sources and compiler settings remain separate",
    );
    if (plan.id.startsWith("original-cli")) assert.equal(plan.checker.state, "UNKNOWN");
    else {
      assert.equal(plan.checker.state, "LAUNCHER_OBSERVED");
      assert(plan.checker.state === "LAUNCHER_OBSERVED");
      const name = plan.id.includes("tsrs")
        ? "tsrs"
        : plan.id === "original-vue-tsc"
          ? "vue-tsc"
          : "typescript";
      const version =
        name === "tsrs" ? recipe.tsrs : name === "vue-tsc" ? recipe.vueTsc : recipe.typescript;
      assert.equal(plan.checker.packageName, name);
      assert.equal(plan.checker.version, version);
      const launcher = plan.checker.executable;
      assert(
        install.packages.some(
          (pkg) =>
            pkg.name === name &&
            pkg.version === version &&
            pkg.files.some(
              (file) =>
                file.path === launcher.path &&
                file.bytes === launcher.bytes &&
                file.sha256 === launcher.sha256,
            ),
        ),
      );
      const observed = object(wholeJson(Buffer.from(plan.checker.versionObservation)));
      assert.equal(observed.status, 0);
      assert.equal(observed.signal, null);
      assert.equal(observed.error, null);
      const expectedVersion =
        name === "tsrs"
          ? "Version 7.1.0-beta (tsrs 0.20.1-ts7.1.0-beta, microsoft/TypeScript@3e9236361fa9)\n"
          : `Version ${recipe.typescript}\n`;
      assert.equal(bytes(text(observed.stdoutBase64)).toString("utf8"), expectedVersion);
      assert.equal(plan.checker.backend, "UNKNOWN");
    }
    const expected = after.fixture.files.map((f) => {
      if (f.path !== "package.json") return f;
      const raw = manifestBytes(
        after.fixture,
        release.version,
        plan.consumer === "tsrs-supplement",
      );
      return { ...f, base64: raw.toString("base64"), sha256: sha256(raw) };
    });
    assert.deepEqual(plan.sourceFiles, expected);
    assert.equal(plan.requiresNativeLoad, plan.id !== "original-vue-tsc");
    const row = capture.observations.find((r) => r.id === plan.id);
    assert(row);
    validateCapture(plan, row, install, false, observers[0]!.path);
    assert.equal(row.instrumentation, "none");
    assert.equal(row.nativeEvents.length, 0);
    if (plan.requiresNativeLoad) {
      companionIds.push(plan.id);
      const companion = capture.companions.find((r) => r.id === plan.id);
      assert(companion);
      validateCapture(plan, companion, install, true, observers[0]!.path);
    }
  }
  assert.deepEqual(
    capture.companions.map((c) => c.id),
    companionIds,
  );
  assert.deepEqual(
    judgment.originalFields,
    capture.observations.map((row) => ({
      id: row.id,
      status: row.status,
      stdoutSha256: sha256(bytes(row.stdoutBase64)),
      stderrSha256: sha256(bytes(row.stderrBase64)),
    })),
  );
  const failure = members.get("failure.json");
  assert(failure);
  const failed = object(wholeJson(failure));
  assert.equal(failed.role, after.role);
  assert.equal(failed.phase, "before");
  assert.equal(failed.accepted, false);
  assert.equal(failed.status, "FAILED");
  assert.equal(failed.error, "Error: public archive and installed package file sets differ");
  assert(
    !members.has("bundle.json") && !members.has("verdict.json"),
    "failed capture is not a strict bundle or verdict",
  );
  const inventories = [...members]
    .filter(([name]) => name.endsWith("-public-package-file-inventory.json"))
    .sort(([a], [b]) => a.localeCompare(b));
  assert(inventories.length > 0);
  const last = object(object(wholeJson(inventories.at(-1)![1])).value);
  assert(text(last.directory).endsWith("/vize"));
  const difference = object(last.difference);
  assert.deepEqual(difference.extraInstalledRegularFiles, [
    "node_modules/.bin/vize",
    "node_modules/.bin/vz",
  ]);
  assert.deepEqual(difference.missingInstalledRegularFiles, []);
}
