import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { Bundle, Fixture, ReleaseAuthority } from "./model.ts";
import { requireComplete } from "./recipes.ts";
import { assertOriginal } from "./fixtures.ts";
import { validateOwnedCommands } from "./commands.ts";
import { validateObservation } from "./processes.ts";
import { validateInstallation } from "./installation.ts";
import { authenticateRelease, type GithubRead } from "./authority.ts";
import { hash, sha256 } from "./value.ts";
import { authenticateController } from "./controller-authority.ts";
import { authenticateSourceAdmission } from "./source-admission.ts";
export async function receive(
  bundle: Bundle,
  expectedFixture: Fixture,
  github: GithubRead,
  archives: ReadonlyMap<string, Buffer>,
  alreadyAuthenticated?: ReleaseAuthority,
): Promise<unknown> {
  const request = bundle.request,
    recipe = requireComplete(request);
  assertOriginal(request.fixture, expectedFixture);
  assert(request.sourceAdmission);
  hash(request.sourceAdmission.commit, 40);
  hash(request.sourceAdmission.receiptSha256);
  const admission = await authenticateSourceAdmission(request, github);
  const controllerAuthority = await authenticateController(
    bundle.controller,
    request.reviewedController,
    github,
    false,
  );
  const authority = alreadyAuthenticated ?? (await authenticateRelease(request.release, github));
  assert.deepEqual(
    bundle.release,
    authority,
    "release evidence must be independently reconstructed",
  );
  hash(bundle.controller.head, 40);
  assert.match(bundle.controller.run, /^[1-9]\d*$/u);
  assert.equal(bundle.controller.job, "public-consumer");
  assert(bundle.controller.files.length > 0);
  for (const image of bundle.controller.files) {
    const raw = readFileSync(image.path);
    assert.equal(raw.length, image.bytes);
    assert.equal(sha256(raw), image.sha256);
  }
  assert.deepEqual(
    bundle.installs.map((v) => v.kind),
    recipe.tsrs ? ["original", "tsrs-supplement"] : ["original"],
  );
  assert.equal(new Set(bundle.installs.map((v) => v.root)).size, bundle.installs.length);
  for (const install of bundle.installs)
    validateInstallation(install, request.fixture, recipe, authority, archives);
  validateOwnedCommands(request, bundle.installs);
  assert.deepEqual(
    bundle.observations.map((v) => v.id),
    request.commands.map((v) => v.id),
  );
  assert.deepEqual(
    bundle.companions?.map((v) => v.id),
    request.commands.filter((v) => v.requiresNativeLoad).map((v) => v.id),
    "exact companion inventory required",
  );
  for (const command of request.commands) {
    const row = bundle.observations.find((v) => v.id === command.id),
      install = bundle.installs.find((v) => v.kind === command.consumer);
    assert(row && install);
    validateObservation(command, row, install);
    if (command.requiresNativeLoad) {
      const companion = bundle.companions?.find((v) => v.id === command.id);
      assert(companion, "separate native-load companion required");
      validateObservation(command, companion, install, true);
    }
  }
  return {
    schema: "vize.humanReports11252_11253.verdict.v1",
    issue: recipe.issue,
    role: recipe.role,
    phase: request.release.phase,
    accepted: request.release.phase === "after",
    beforeObservationOnly: request.release.phase === "before",
    authority,
    installations: bundle.installs,
    observations: bundle.observations,
    controller: bundle.controller,
    controllerAuthority,
    sourceAdmission: admission,
    publicArchiveBodiesUploaded: false,
    nativeImagesUploaded: false,
    authenticatedInstalledPackagePayloadsRechecked: true,
    wholeNodeModulesFilesystemImmutabilityClaim: false,
    scope:
      "Only this issue-bound original recipe and explicit finite controls; no broader library, WSL2, backend, or latency claim",
  };
}
