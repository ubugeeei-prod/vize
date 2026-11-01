import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { inside } from "../computed_event_custody.ts";
import type {
  Bundle,
  CaptureInstallReceipt,
  CommandPlan,
  Fixture,
  InstallationFacts,
  InstallReceipt,
  Observation,
  Request,
  SourceFile,
} from "./model.ts";
import { authenticateRelease } from "./authority.ts";
import { beforeCommands, requireCaptureReady } from "./recipes.ts";
import { assertOriginal, validateFiles } from "./fixtures.ts";
import { argumentsFor, validateOwnedCommands } from "./commands.ts";
import { preparePublic, recheckInstalled } from "./installer.ts";
import { manifestBytes } from "./installation.ts";
import { githubReader } from "./network.ts";
import { recordCommand } from "./capture.ts";
import { identifyLauncher } from "./checker.ts";
import { receive } from "./receive.ts";
import { authenticatePublicBefore } from "./public-before.ts";
import { authenticateCapturedBefore } from "./public-before-capture.ts";
import { controllerFiles } from "./controller.ts";
import { validateCapture } from "./processes.ts";
import { evidenceSink } from "./evidence.ts";
import { authenticateController } from "./controller-authority.ts";
import { authenticateSourceAdmission } from "./source-admission.ts";
import { mainManualContext } from "./context.ts";
import { inputWitness, payloadWitness } from "./witness.ts";
import { bytes, sha256 } from "./value.ts";
import { afterPlan, inputCommand, unqualifiedAfterVerdict } from "./after-plan.ts";
import { authenticateAfterInputs } from "./after-admission.ts";
import {
  assertInputState,
  checkCaseManagers,
  inputStateWitness,
  restoreInputState,
} from "./input-state.ts";
import { managerFiles } from "./optional-residency.ts";
import { controllerBinPolicy } from "./dependency-bin-policy.ts";
function file(path: string, raw: Buffer): SourceFile {
  return { path, base64: raw.toString("base64"), sha256: sha256(raw) };
}
function sourceInputs(fixture: Fixture, version: string, supplement: boolean): SourceFile[] {
  return fixture.files.map((f) =>
    f.path === "package.json" ? file(f.path, manifestBytes(fixture, version, supplement)) : f,
  );
}
function checkInputs(command: CommandPlan, root: string): void {
  for (const source of command.sourceFiles)
    assert(readFileSync(join(root, source.path)).equals(bytes(source.base64)));
}
export function retainCaptureBeforeQualification(
  capture: {
    request: Request;
    release: Bundle["release"];
    installs: CaptureInstallReceipt[];
    observations: Observation[];
    companions: Observation[];
    controller: Bundle["controller"];
  },
  save: (value: unknown) => void,
  qualify: () => InstallReceipt[],
): InstallReceipt[] {
  for (const install of capture.installs)
    assert.equal(install.schema, "vize.humanReports11252_11253.capturePrerequisites.v1");
  save({
    ...capture,
    schema: "vize.humanReports11252_11253.unqualifiedCapture.v1",
    completeInstalledFileSetQualified: false,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
    outcomeAssessed: false,
  });
  const qualified = qualify();
  assert.deepEqual(
    qualified.map((i) => i.kind),
    capture.installs.map((i) => i.kind),
  );
  for (const install of qualified) {
    assert.equal(
      install.schema,
      "vize.humanReports11252_11253.install.v1",
      "pending capture cannot become a qualified installation",
    );
    assert(
      !("completeInstalledFileSetQualified" in install),
      "pending verdict cannot cross strict egress",
    );
    assert(install.packages.length > 0);
    for (const pkg of install.packages) {
      assert(
        !("completeInstalledFileSetQualified" in pkg),
        "pending package cannot cross strict egress",
      );
      assert(typeof pkg.allInstalledBytesEqualArchive === "boolean");
    }
  }
  return qualified;
}
export async function runPublic(
  request: Request,
  fixture: Fixture,
  pnpmCli: string,
  output: string,
): Promise<void> {
  mainManualContext();
  const recipe = requireCaptureReady(request);
  assertOriginal(request.fixture, fixture);
  validateFiles(fixture.files);
  const toolRoot = dirname(fileURLToPath(import.meta.url)),
    canonicalParent = realpathSync(dirname(resolve(output)));
  assert(!inside(resolve(toolRoot, "../../../../"), canonicalParent));
  mkdirSync(output);
  output = realpathSync(output);
  const write = (name: string, value: unknown) =>
    writeFileSync(join(output, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
  const running = {
    schema: "vize.humanReports11252_11253.execution.v1",
    role: request.role,
    phase: request.release.phase,
    accepted: false,
    ...(request.mode === "UNQUALIFIED_AFTER"
      ? {
          productQualified: false,
          afterQualified: false,
          issueQualified: false,
          outcomeAssessed: false,
          knownBugReproductionJudged: false,
          backend: "UNKNOWN",
        }
      : {}),
    status: "RUNNING",
  };
  write("execution-start.json", running);
  const toolFiles = controllerFiles();
  const checkController = () =>
    assert.deepEqual(
      controllerFiles(),
      toolFiles,
      "controller/generic helper bytes changed; execute no further command",
    );
  const sink = evidenceSink(output),
    github = githubReader(sink);
  const controller = {
    head: process.env.GITHUB_SHA ?? "",
    run: process.env.GITHUB_RUN_ID ?? "",
    job: process.env.GITHUB_JOB ?? "",
    files: toolFiles,
  };
  try {
    write(
      "controller-authority.json",
      await authenticateController(controller, request.reviewedController, github, false),
    );
    const binPolicy = controllerBinPolicy(request.reviewedController);
    write("controller-bin-input-policy.json", binPolicy);
    const release = await authenticateRelease(request.release, github, sink);
    write("release-authority.json", release);
    if (request.release.phase === "after")
      write(
        request.mode === "UNQUALIFIED_AFTER"
          ? "authenticated-before-capture.json"
          : "authenticated-public-before.json",
        request.mode === "UNQUALIFIED_AFTER"
          ? await authenticateCapturedBefore(request, release, github, sink)
          : await authenticatePublicBefore(request, github, sink),
      );
    if (request.release.phase === "after")
      write(
        "source-admission.json",
        request.mode === "UNQUALIFIED_AFTER"
          ? await authenticateAfterInputs(request, release, github)
          : await authenticateSourceAdmission(request, github),
      );
    const archiveCache = new Map<string, Buffer>();
    checkController();
    const prepared = [
      await preparePublic(
        join(output, "consumer-original"),
        "original",
        fixture,
        recipe,
        release,
        pnpmCli,
        output,
        archiveCache,
        sink,
        binPolicy,
      ),
    ];
    if (recipe.tsrs)
      prepared.push(
        await preparePublic(
          join(output, "consumer-tsrs-supplement"),
          "tsrs-supplement",
          fixture,
          recipe,
          release,
          pnpmCli,
          output,
          archiveCache,
          sink,
          binPolicy,
        ),
      );
    // A pending capture installation is never an InstallReceipt or a Bundle.
    // Strict acceptance still qualifies before execution. Observation-only
    // modes preserve complete raw vectors before the final complete-set guard.
    const qualifiedBeforeCommands =
      request.mode === "ACCEPT_AFTER"
        ? prepared.map((installation) => installation.qualify())
        : null;
    const installs: InstallationFacts[] = qualifiedBeforeCommands ?? prepared.map((p) => p.capture);
    const recheck = (install: InstallationFacts, expectedManifest?: Buffer) => {
      const current = prepared.find((row) => row.capture.kind === install.kind);
      assert(current);
      current.recheck(expectedManifest);
      if (qualifiedBeforeCommands) {
        const qualified = qualifiedBeforeCommands.find((row) => row.kind === install.kind);
        assert(qualified);
        recheckInstalled(qualified, expectedManifest);
      }
    };
    if (request.release.phase === "before") {
      checkController();
      request.commands = beforeCommands[request.role].map((id): CommandPlan => {
        const consumer = id.includes("tsrs") ? "tsrs-supplement" : "original";
        const install = installs.find((v) => v.kind === consumer);
        assert(install);
        const sourceFiles = sourceInputs(fixture, release.version, consumer === "tsrs-supplement");
        return {
          id,
          consumer,
          executable: install.pnpm.path,
          argv: argumentsFor(request, id),
          sourceFiles,
          purpose:
            id === "original-pnpm-check" ||
            (request.role === "imports-vue-ts-11253" && id !== "original-cli-json")
              ? "original"
              : "supplement",
          adaptation:
            id === "original-pnpm-check" ||
            (request.role === "imports-vue-ts-11253" && id !== "original-cli-json")
              ? null
              : "Explicit report-note/supplement command; original script, sources and compiler settings remain separate",
          checker: identifyLauncher(id, install, output, sink),
          requiresNativeLoad: id !== "original-vue-tsc",
          oracle: null,
        };
      });
    } else {
      checkController();
      for (const command of request.commands) {
        const install = installs.find((v) => v.kind === command.consumer);
        assert(install);
        command.executable = install.pnpm.path;
        command.argv = argumentsFor(request, command.id);
        command.checker = identifyLauncher(command.id, install, output, sink);
      }
      if (request.mode === "UNQUALIFIED_AFTER")
        assert.deepEqual(
          request.commands.map(inputCommand),
          afterPlan(request.role, release.version).map((p) => inputCommand(p.command)),
        );
      else validateOwnedCommands(request, installs);
    }
    write("effective-request.json", request);
    const observations: Observation[] = [],
      companions: Observation[] = [];
    const after =
      request.mode === "UNQUALIFIED_AFTER" ? afterPlan(request.role, release.version) : null;
    // Complete the entire original uninstrumented vector before any companion
    // preload. All authentic installed payloads are rechecked around each run.
    for (const companion of [false, true])
      for (const command of request.commands) {
        if (companion && !command.requiresNativeLoad) continue;
        const install = installs.find((v) => v.kind === command.consumer);
        assert(install);
        const planned = after?.find((p) => p.command.id === command.id);
        const inputSink = (kind: string, value: unknown) =>
          sink(kind, {
            role: request.role,
            commandId: command.id,
            consumer: command.consumer,
            companion,
            value,
          });
        if (planned && after) {
          restoreInputState(
            install.root,
            planned.before,
            after
              .filter((p) => p.command.consumer === command.consumer)
              .flatMap((p) => [p.before, p.after]),
            inputSink,
            (current) => {
              const rootPackage = current.present.find((f) => f.path === "package.json");
              assert(rootPackage);
              checkCaseManagers(
                install.managerFiles,
                managerFiles(install.root),
                bytes(rootPackage.base64),
              );
            },
          );
          inputSink(
            "complete-input-before-process",
            inputStateWitness(install.root, planned.before),
          );
        }
        sink("pre-payload-witness", payloadWitness(install));
        checkController();
        const beforePackage = planned?.before.present.find((f) => f.path === "package.json");
        recheck(install, beforePackage ? bytes(beforePackage.base64) : undefined);
        const row = recordCommand(command, install, output, companion, sink, Boolean(planned));
        (companion ? companions : observations).push(row);
        write(`${command.id}.${companion ? "companion" : "original"}.observation.json`, row);
        sink("actual-input-after-process", inputWitness(command, install.root));
        if (planned) {
          inputSink("complete-input-after-process", inputStateWitness(install.root, planned.after));
          assertInputState(install.root, planned.after);
        }
        sink("post-payload-witness", payloadWitness(install));
        checkInputs(command, install.root);
        const afterPackage = planned?.after.present.find((f) => f.path === "package.json");
        recheck(install, afterPackage ? bytes(afterPackage.base64) : undefined);
      }
    assert.deepEqual(
      controllerFiles(),
      toolFiles,
      "controller/generic helper bytes changed during public collection",
    );
    const qualified =
      qualifiedBeforeCommands ??
      retainCaptureBeforeQualification(
        {
          request,
          release,
          installs: prepared.map((p) => p.capture),
          observations,
          companions,
          controller,
        },
        (raw) => write("unqualified-capture.json", raw),
        () => prepared.map((installation) => installation.qualify()),
      );
    const bundle: Bundle = {
      request,
      release,
      installs: qualified,
      observations,
      companions,
      controller,
    };
    write("bundle.json", bundle);
    if (request.release.phase === "before") {
      for (const plan of request.commands) {
        const original = observations.find((v) => v.id === plan.id),
          install = installs.find((v) => v.kind === plan.consumer);
        assert(original && install);
        validateCapture(plan, original, install);
        if (plan.requiresNativeLoad) {
          const companion = companions.find((v) => v.id === plan.id);
          assert(companion);
          validateCapture(plan, companion, install, true);
        }
      }
      write("verdict.json", {
        ...running,
        mode: "CAPTURE_BEFORE",
        status: "OBSERVED_PUBLIC_BEFORE_ONLY",
        accepted: false,
        productQualified: false,
        afterQualified: false,
        issueQualified: false,
        knownBugReproductionJudged: false,
        originalAndCompanionKeptSeparate: true,
        sourceBeforeTransferred: false,
        backend: "UNKNOWN",
      });
    } else if (request.mode === "UNQUALIFIED_AFTER") {
      assert.deepEqual(
        observations.map((r) => r.id),
        request.commands.map((c) => c.id),
      );
      assert.deepEqual(
        companions.map((r) => r.id),
        request.commands.filter((c) => c.requiresNativeLoad).map((c) => c.id),
      );
      for (const plan of request.commands) {
        const install = installs.find((v) => v.kind === plan.consumer);
        const original = observations.find((r) => r.id === plan.id);
        assert(install && original);
        validateCapture(plan, original, install);
        if (plan.requiresNativeLoad) {
          const companion = companions.find((r) => r.id === plan.id);
          assert(companion);
          validateCapture(plan, companion, install, true);
        }
      }
      write("verdict.json", unqualifiedAfterVerdict(request.role));
    } else write("verdict.json", await receive(bundle, fixture, github, archiveCache, release));
    // Only records/declarations/virtual source are proof uploads. The consumer
    // directories, public tarball buffers and native binary bodies are excluded.
  } catch (error) {
    write("failure.json", { ...running, status: "FAILED", error: String(error) });
    throw error;
  }
}
