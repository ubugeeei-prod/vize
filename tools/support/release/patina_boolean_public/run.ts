import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { asRecord, asString } from "../../json-value.ts";
import { Recorder, sha256, success } from "../../config/windows_consumer/common.ts";
import { bindLocalNpx, npxCacheSnapshot } from "../editor_vite_stub/config.ts";
import { observed, verifyJournal, hook } from "../editor_vite_stub/custody.ts";
import { capture, completed } from "../constructor_original27_public/capture.ts";
import { authenticate } from "./authority.ts";
import { identity, install, provider } from "./install.ts";
import {
  assertHost,
  assertJournalPid,
  assertReport,
  originalFiles,
  readOriginal,
  rejectOverrides,
  requiredMerge,
  type Phase,
} from "./contract.ts";
import { captureSelectedRuntime } from "./observe.ts";

export async function main(args: readonly string[]): Promise<void> {
  assertHost();
  rejectOverrides();
  assert.equal(
    process.env.GITHUB_ACTIONS,
    "true",
    "public native execution belongs to the single hosted owner",
  );
  assert.equal(process.env.GITHUB_REF, "refs/heads/main", "merged witness source required");
  assert.equal(args.length, 4, "version producer-run capsule-blob output required");
  const [version, run, blob, output] = args;
  assert(version && run && blob && output);
  const directory = resolve(output);
  assert(
    !directory.startsWith(realpathSync(process.cwd()) + "/"),
    "fresh consumer must be outside the checkout",
  );
  mkdirSync(directory, { recursive: false });
  const record = new Recorder(realpathSync(directory));
  const original = readOriginal();
  record.json("original-issue-fixture.json", original);
  const authority = await authenticate(version, run, blob, record);
  const root = join(record.root, "consumer"),
    tooling = join(record.root, "package-manager");
  const pnpm = install(root, tooling, version, original, record);
  const phaseResults: unknown[] = [];
  let stableRaw: Buffer | undefined;
  let originalIdentity: { cli: string; native: string } | undefined;
  for (const phase of ["original-no-provider", "stable-user-provider", "reported-tsrs"] as const) {
    if (phase === "stable-user-provider") {
      success(
        record.run(
          "install-explicit-stable-user-provider",
          [...pnpm, "add", "-D", "--save-exact", "@typescript/typescript-darwin-arm64@7.0.2"],
          root,
          process.env,
          600_000,
        ),
      );
    } else if (phase === "reported-tsrs") {
      success(
        record.run(
          "remove-stable-with-package-manager",
          [...pnpm, "remove", "-D", "@typescript/typescript-darwin-arm64"],
          root,
          process.env,
          600_000,
        ),
      );
      success(
        record.run(
          "install-original-tsrs",
          [...pnpm, "add", "-D", "--save-exact", "tsrs@0.20.1-ts7.1.0-beta"],
          root,
          process.env,
          600_000,
        ),
      );
    }
    const installed = identity(
      root,
      phase,
      version,
      authority.capsule,
      authority.candidate,
      run,
      record,
    );
    const binary = asRecord(installed.binary);
    const currentIdentity = {
      cli: asString(asRecord(installed.cliWrapper).sha256),
      native: asString(binary.sha256),
    };
    if (originalIdentity)
      assert.deepEqual(
        currentIdentity,
        originalIdentity,
        "same public Vize source/image across package-manager phases",
      );
    else originalIdentity = currentIdentity;
    const executable = provider(root, phase, record);
    const npx = bindLocalNpx(
      root,
      { packageName: "vize", bin: "vize", version, entry: asString(installed.cli) },
      phaseRecorder(record, phase),
    );
    const originalRun = capture(
      record,
      phase + "-original-lint",
      [...npx.command, "vize", "lint", "--type-aware", "src/card.vue"],
      root,
      npx.env,
    );
    completed(originalRun, 1, false);
    record.json(phase + ".npx-cache.json", npxCacheSnapshot(npx.cache));
    const custody = observed(root, installed, record, phase + "-json-cli");
    const argv = [
      process.execPath,
      "--require",
      hook,
      asString(installed.cli),
      "lint",
      "--type-aware",
      "--format",
      "json",
      "src/card.vue",
    ];
    const report = executable
      ? await captureSelectedRuntime(
          record,
          phase + "-complete-report",
          argv,
          root,
          custody.env,
          executable,
        )
      : capture(record, phase + "-complete-report", argv, root, custody.env);
    completed(report, 1);
    verifyJournal(custody.journal, installed, report.status);
    assertJournalPid(readFileSync(custody.journal), report.pid);
    assertReport(original, phase, report.stdout);
    if (phase === "stable-user-provider") stableRaw = report.stdout;
    if (phase === "reported-tsrs") {
      assert(stableRaw);
      assert(
        report.stdout.equals(stableRaw),
        "all original JSON bytes, framing and report fields match the explicitly named stable-provider control",
      );
      const check = capture(
        record,
        phase + "-original-check",
        [...npx.command, "vize", "check", "src"],
        root,
        npx.env,
      );
      completed(check, 0, false);
    }
    for (const [name, content] of originalFiles(original))
      assert(
        readFileSync(join(root, name)).equals(Buffer.from(content)),
        "unchanged whole authored input: " + name,
      );
    phaseResults.push({
      phase,
      originalCommand: originalRun.label,
      completeReport: report.label,
      completeReportSha256: sha256(report.stdout),
      selectedExecutable: executable ?? null,
      nativeJournal: custody.journal,
    });
  }
  record.json("qualification.json", {
    schema: "vize.patina-original-11147-public.v1",
    issue: 11147,
    version,
    releaseSource: authority.candidate,
    producerSource: authority.ownership.producerSource,
    requiredMerge,
    producerRun: run,
    authorityBlob: blob,
    workflowSource: process.env.GITHUB_SHA,
    workflowRun: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    host: { platform: process.platform, arch: process.arch, node: process.version, pnpm: "12.1.0" },
    phaseResults,
    limits: [
      "One original macOS ARM64/Node26/pnpm role, not a platform matrix. Node26.6.0/pnpm12.1.0/TypeScript6.0.3 are explicit pins for unspecified patches/hosts.",
      "The original dependency recipe now has no native provider under current explicit-user-provider policy. Its complete error is retained, not replaced by the stable-provider control or represented as the reporter's successful baseline.",
      "Stable user provider and tsrs-only actual reports must match all stdout bytes. ps/lsof observes actual selected text-image/PID separately; native-loader journal binds actual public Vize image. No raw protocol packet capture or direct SDK self-introspection.",
      "The original92-SFC repeated-warning population and unreproduced~620-SFC symbol-owner failure remain unqualified. No synthetic clone population, heap/timeout diagnosis or Issue closure follows.",
    ],
  });
}

/** Isolate generic npx helper outputs without altering the real consumer. */
function phaseRecorder(record: Recorder, phase: Phase): Recorder {
  return new Recorder(join(record.root, phase + "-npx"));
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main(process.argv.slice(2));
