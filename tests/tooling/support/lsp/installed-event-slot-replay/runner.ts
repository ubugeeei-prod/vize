import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { inspect } from "node:util";
import type { VizePublicRegistryInstallAuthority } from "../installed-alias-replay/authority-schema.ts";
import { recheckPublicBytes } from "../installed-alias-replay/authority.ts";
import {
  digest,
  publicEnvironment,
  verifyNativeJournal,
} from "../installed-alias-replay/custody.ts";
import { InstalledAliasSession } from "../installed-alias-replay/protocol.ts";
import { auditEventGuard, observeEventGuard, type EventGuardRecord } from "./runner-guard.ts";
import { loadReplayCases, publicDomLibrary, type ControllerRecord } from "./runner-scopes.ts";

type ReplayOptions = {
  authority: VizePublicRegistryInstallAuthority;
  sourceRoot: string;
  vuePath: string;
  vueCustody: unknown;
  recheckVue: () => void;
  outputRoot: string;
};
const errorRecord = (error: unknown) => ({
  detail: inspect(error, { depth: null, maxArrayLength: null, maxStringLength: null }),
  ...(error instanceof Error
    ? {
        name: error.name,
        message: error.message,
        stack: error.stack,
        cause: inspect(error.cause, { depth: null, maxArrayLength: null, maxStringLength: null }),
      }
    : {}),
});

/** Exactly60 real public sessions. No source fallback, provider simulation or issue closure. */
export async function replayInstalledEventSlotScopes(options: ReplayOptions) {
  assert.ok(
    path.isAbsolute(options.outputRoot) && !fs.existsSync(options.outputRoot),
    "fresh replay output required",
  );
  assert.equal(
    path.join(fs.realpathSync(path.dirname(options.outputRoot)), path.basename(options.outputRoot)),
    options.outputRoot,
    "replay output must use its canonical physical parent",
  );
  options.recheckVue();
  recheckPublicBytes(options.authority);
  assert.equal(fs.realpathSync(options.vuePath), options.vuePath);
  assert.ok(fs.statSync(path.join(options.vuePath, "package.json")).isFile());
  const libraryPath = publicDomLibrary(options.authority);
  const cases = loadReplayCases(options.sourceRoot, libraryPath);
  fs.mkdirSync(options.outputRoot);
  const outcomes: Array<Record<string, unknown>> = [];
  const receipt = {
    schema: "vize-installed-original-event-slot-scopes-v1",
    authority: options.authority,
    vueCustody: options.vueCustody,
    sessions: { originalEventsAndSlots: 18, library: 22, bound: 20, total: 60 },
    libraryCustody: { path: libraryPath, sha256: digest(fs.readFileSync(libraryPath)) },
    scope:
      "The additional14 source-native ownership/refusal sessions remain pending separately; successful60 does not close #8010 or #8011.",
    adaptations: [
      "The original explicit workspace Corsa path is absent; the authentic npm CLI selects its public bundled Corsa.",
      "Original18 retain their complete flat transaction and request IDs; a separate authored TS2322 invalid/repair witness uses notifications only.",
      "All original tsconfig, source/golden bytes, lint settings, whole packets and twenty-second message/five-second stdin-open exit deadlines remain explicit.",
    ],
    outcomes,
  };
  const persist = () =>
    fs.writeFileSync(
      path.join(options.outputRoot, "receipt.json"),
      `${JSON.stringify(receipt, null, 2)}\n`,
    );
  persist();
  for (const testCase of cases) {
    const caseRoot = path.join(options.outputRoot, testCase.id);
    const projectRoot = path.join(caseRoot, "project");
    fs.mkdirSync(projectRoot, { recursive: true });
    fs.mkdirSync(path.join(projectRoot, "node_modules"));
    const vueLink = path.join(projectRoot, "node_modules/vue");
    fs.symlinkSync(options.vuePath, vueLink, "dir");
    assert.equal(fs.realpathSync(vueLink), options.vuePath);
    const scope = testCase.prepare(projectRoot);
    fs.writeFileSync(path.join(projectRoot, "tsconfig.json"), scope.tsconfig);
    fs.writeFileSync(path.join(projectRoot, "vize.config.json"), JSON.stringify(scope.vizeConfig));
    for (const file of scope.files) {
      assert.ok(!path.isAbsolute(file.file) && !file.file.split(/[\\/]/u).includes(".."));
      const target = path.join(projectRoot, file.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, file.text);
    }
    assert.equal(fs.realpathSync(projectRoot), projectRoot);
    // Complete independent expectations and guard are durable before launching the provider.
    const expectedPath = path.join(caseRoot, "expected.json");
    fs.writeFileSync(
      expectedPath,
      `${JSON.stringify({ controller: scope.expected, guard: scope.guard }, null, 2)}\n`,
    );
    const expectedSha256 = digest(fs.readFileSync(expectedPath));
    const journalPath = path.join(caseRoot, "native-loader.ndjson");
    const outcome: Record<string, unknown> = {
      id: testCase.id,
      family: testCase.family,
      context: testCase.context,
      phase: "prepared",
      status: "running",
      projectRoot,
      source: scope.files,
      configuration: { tsconfig: scope.tsconfig, vizeConfig: scope.vizeConfig },
      expectedPath,
      expectedSha256,
      journalPath,
      expected: scope.expected,
      ...(scope.guard ? { separateGuardExpectation: scope.guard } : {}),
      ...(scope.pendingBatchWitness ? { pendingBatchWitness: scope.pendingBatchWitness } : {}),
      vueLink: { path: vueLink, target: options.vuePath },
      deadlines: { messageMs: 20_000, exitMs: 5_000, stdinRemainsOpenAfterExit: true },
      shutdownId: scope.shutdownId,
    };
    outcomes.push(outcome);
    persist();
    let session: InstalledAliasSession | undefined;
    let record: ControllerRecord | undefined;
    let guard: EventGuardRecord | undefined;
    const capture = (phase: string) => {
      outcome.phase = phase;
      if (session) outcome.transport = session.receipt();
      persist();
      fs.writeFileSync(
        path.join(caseRoot, "phase-observations.json"),
        `${JSON.stringify(outcome, null, 2)}\n`,
      );
    };
    try {
      recheckPublicBytes(options.authority);
      options.recheckVue();
      capture("launching-public-provider");
      session = await InstalledAliasSession.launch({
        nodePath: options.authority.node.path,
        cliPath: options.authority.cli.binPath,
        custodyHookPath: options.authority.custodyHook.path,
        projectRoot,
        env: publicEnvironment(options.authority, journalPath),
        outputRoot: caseRoot,
        timeoutMs: 20_000,
        crossFile: true,
        lint: scope.lint,
        keepStdinOpenAfterExit: true,
        exitTimeoutMs: 5_000,
      });
      capture("initialized");
      if (scope.guard) {
        guard = await observeEventGuard(session, projectRoot, scope.guard, (partial) => {
          guard = partial;
          outcome.separateGuard = partial;
          capture("separate-native-guard");
        });
        capture("separate-native-guard-complete");
      }
      record = await scope.observe(session, (partial) => {
        record = partial;
        outcome.controller = partial;
        capture("original-controller-observation");
      });
      capture("original-controller-complete");
    } catch (error) {
      outcome.error = errorRecord(error);
      capture("observation-failed");
    } finally {
      if (session) {
        capture("shutdown-started");
        try {
          outcome.terminal = await session.shutdown();
        } catch (error) {
          outcome.shutdownError = errorRecord(error);
        }
        capture("shutdown-complete");
        try {
          if (record) outcome.finalControllerFailures = scope.audit(record, session.notifications);
          if (guard) outcome.finalGuardFailures = auditEventGuard(guard, session.notifications);
        } catch (error) {
          outcome.auditError = errorRecord(error);
        }
        capture("late-publications-audited");
      }
      try {
        const directories = fs.readdirSync(caseRoot).filter((name) => name.startsWith("session-"));
        outcome.transportDirectories = directories.map((name) => path.join(caseRoot, name));
        outcome.transportCaptures = directories.map((name) => {
          const directory = path.join(caseRoot, name);
          return {
            directory,
            receipt: JSON.parse(fs.readFileSync(path.join(directory, "receipt.json"), "utf8")),
            raw: ["client", "server", "stderr"].map((stream) => {
              const file = path.join(directory, `${stream}.raw`);
              const bytes = fs.readFileSync(file);
              return { file, bytes: bytes.length, sha256: digest(bytes) };
            }),
          };
        });
      } catch (error) {
        outcome.transportCaptureError = errorRecord(error);
      }
      const custodyChecks: Array<Record<string, unknown>> = [];
      const checks: Array<[string, () => unknown]> = [
        [
          "native-loader-return",
          () => {
            outcome.nativeCustody = verifyNativeJournal(
              journalPath,
              options.authority,
              session?.receipt().pid ?? undefined,
            );
          },
        ],
        ["whole-public-payload", () => recheckPublicBytes(options.authority)],
        ["whole-stock-vue-graph", () => options.recheckVue()],
        [
          "independent-pre-query-oracle",
          () =>
            assert.equal(
              digest(fs.readFileSync(expectedPath)),
              expectedSha256,
              "pre-query oracle bytes changed",
            ),
        ],
        ["approved-stock-vue-link", () => assert.equal(fs.realpathSync(vueLink), options.vuePath)],
      ];
      // A failed loader journal cannot suppress the independent post-session byte checks.
      for (const [name, check] of checks) {
        try {
          check();
          custodyChecks.push({ name, success: true });
        } catch (error) {
          custodyChecks.push({ name, success: false, error: errorRecord(error) });
        }
        outcome.postSessionCustody = custodyChecks;
        capture(`post-session-${name}`);
      }
      const custodyErrors = custodyChecks.filter((check) => check.success === false);
      if (custodyErrors.length) outcome.custodyError = custodyErrors;
      capture("post-session-custody-complete");
    }
    try {
      for (const key of [
        "error",
        "shutdownError",
        "auditError",
        "custodyError",
        "transportCaptureError",
      ])
        assert.equal(outcome[key], undefined, `complete ${key} retained in receipt`);
      assert.ok(session && record);
      const terminal = session.receipt();
      assert.equal(terminal.closed, true);
      assert.equal(terminal.exitCode, 0);
      assert.equal(terminal.signal, null);
      assert.equal(terminal.parsingFailed, false);
      assert.equal(terminal.remainingFrameBytes, 0);
      assert.deepEqual(terminal.failures, []);
      assert.deepEqual(
        terminal.packets.filter((packet) => packet.id === scope.shutdownId),
        [{ jsonrpc: "2.0", id: scope.shutdownId, result: null }],
        "whole original shutdown response",
      );
      assert.deepEqual(record.failures, []);
      assert.deepEqual(outcome.finalControllerFailures, []);
      if (scope.guard) {
        assert.ok(guard);
        assert.deepEqual(guard.failures, []);
        assert.deepEqual(outcome.finalGuardFailures, []);
      }
      outcome.status = "success";
    } catch (error) {
      outcome.status = "failure";
      outcome.assertion = errorRecord(error);
    }
    capture("complete");
  }
  assert.equal(outcomes.length, 60);
  assert.ok(
    outcomes.every((outcome) => outcome.status === "success"),
    `installed original60 failed; complete receipt ${options.outputRoot}`,
  );
  return receipt;
}
