import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { inside } from "../computed_event_custody.ts";
import type { Bundle, CommandPlan, Fixture, Observation, Request, SourceFile } from "./model.ts";
import { authenticateRelease } from "./authority.ts";
import { beforeCommands, requireCaptureReady } from "./recipes.ts";
import { assertOriginal, validateFiles } from "./fixtures.ts";
import { argumentsFor, validateOwnedCommands } from "./commands.ts";
import { installPublic, recheckInstalled } from "./installer.ts";
import { manifestBytes } from "./installation.ts";
import { githubReader } from "./network.ts";
import { recordCommand } from "./capture.ts";
import { identifyLauncher } from "./checker.ts";
import { receive } from "./receive.ts";
import { authenticatePublicBefore } from "./public-before.ts";
import { controllerFiles } from "./controller.ts";
import { validateCapture } from "./processes.ts";
import { evidenceSink } from "./evidence.ts";
import { authenticateController } from "./controller-authority.ts";
import { authenticateSourceAdmission } from "./source-admission.ts";
import { mainManualContext } from "./context.ts";
import { inputWitness, payloadWitness } from "./witness.ts";
import { bytes, sha256 } from "./value.ts";
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
    status: "RUNNING",
  };
  write("execution-start.json", running);
  const toolFiles = controllerFiles();
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
    const release = await authenticateRelease(request.release, github, sink);
    write("release-authority.json", release);
    if (request.release.phase === "after")
      write(
        "authenticated-public-before.json",
        await authenticatePublicBefore(request, github, sink),
      );
    if (request.release.phase === "after")
      write("source-admission.json", await authenticateSourceAdmission(request, github));
    const archiveCache = new Map<string, Buffer>();
    const installs = [
      await installPublic(
        join(output, "consumer-original"),
        "original",
        fixture,
        recipe,
        release,
        pnpmCli,
        output,
        archiveCache,
        sink,
      ),
    ];
    if (recipe.tsrs)
      installs.push(
        await installPublic(
          join(output, "consumer-tsrs-supplement"),
          "tsrs-supplement",
          fixture,
          recipe,
          release,
          pnpmCli,
          output,
          archiveCache,
          sink,
        ),
      );
    if (request.release.phase === "before") {
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
          checker: identifyLauncher(id, install, output),
          requiresNativeLoad: id !== "original-vue-tsc",
          oracle: null,
        };
      });
    } else {
      for (const command of request.commands) {
        const install = installs.find((v) => v.kind === command.consumer);
        assert(install);
        command.executable = install.pnpm.path;
        command.argv = argumentsFor(request, command.id);
        command.checker = identifyLauncher(command.id, install, output);
      }
      validateOwnedCommands(request, installs);
    }
    write("effective-request.json", request);
    const observations: Observation[] = [],
      companions: Observation[] = [];
    // Complete the entire original uninstrumented vector before any companion
    // preload. All authentic installed payloads are rechecked around each run.
    for (const companion of [false, true])
      for (const command of request.commands) {
        if (companion && !command.requiresNativeLoad) continue;
        const install = installs.find((v) => v.kind === command.consumer);
        assert(install);
        recheckInstalled(install);
        sink("pre-payload-witness", payloadWitness(install));
        const row = recordCommand(command, install, output, companion, sink);
        (companion ? companions : observations).push(row);
        write(`${command.id}.${companion ? "companion" : "original"}.observation.json`, row);
        sink("actual-input-after-process", inputWitness(command, install.root));
        sink("post-payload-witness", payloadWitness(install));
        checkInputs(command, install.root);
        recheckInstalled(install);
      }
    assert.deepEqual(
      controllerFiles(),
      toolFiles,
      "controller/generic helper bytes changed during public collection",
    );
    const bundle: Bundle = { request, release, installs, observations, companions, controller };
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
    } else write("verdict.json", await receive(bundle, fixture, github, archiveCache, release));
    // Only records/declarations/virtual source are proof uploads. The consumer
    // directories, public tarball buffers and native binary bodies are excluded.
  } catch (error) {
    write("failure.json", { ...running, status: "FAILED", error: String(error) });
    throw error;
  }
}
