import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { inspect } from "node:util";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { recheckPublicBytes } from "./authority.ts";
import { loadFrozenAliasCases, rebindFrozenExpected } from "./cases.ts";
import { digest, exactFile, publicEnvironment, verifyNativeJournal } from "./custody.ts";
import { InstalledAliasSession } from "./protocol.ts";
import { observeFrozenCase } from "./transactions.ts";
import { vueConfigFixtureSource } from "../../vue-config-fixture.ts";

type ReplayOptions = {
  authority: VizePublicRegistryInstallAuthority;
  fixtureRoot: string;
  vuePath: string;
  vueCustody: unknown;
  recheckVue: () => void;
  outputRoot: string;
  timeoutMs: number;
};
const tsconfig = `{
            "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
            "include": ["*.vue"]
        }`;
const vizeConfig = {
  experimentals: { patternedTemplate: false },
  typeChecker: { checkFallthroughAttrs: false, optionsApi: false },
  lsp: { lint: false, typecheck: true, hover: true, crossFile: true },
};
const nativeConfigSource = JSON.stringify(vizeConfig);
const configSource = vueConfigFixtureSource(nativeConfigSource);

/** No public identity is assigned here; an authenticated installed authority is mandatory. */
export async function replayInstalledAliases(options: ReplayOptions) {
  assert.ok(
    path.isAbsolute(options.outputRoot) && !fs.existsSync(options.outputRoot),
    "fresh replay output required",
  );
  assert.equal(
    path.join(fs.realpathSync(path.dirname(options.outputRoot)), path.basename(options.outputRoot)),
    options.outputRoot,
    "replay output must use the canonical parent path",
  );
  options.recheckVue();
  assert.equal(fs.realpathSync(options.vuePath), options.vuePath);
  assert.ok(fs.statSync(path.join(options.vuePath, "package.json")).isFile());
  const cases = loadFrozenAliasCases(options.fixtureRoot);
  fs.mkdirSync(options.outputRoot);
  const outcomes: Array<Record<string, unknown>> = [];
  const receipt = {
    schema: "vize-installed-original-alias-transactions-v1",
    authority: options.authority,
    vueCustody: options.vueCustody,
    cases: 32,
    outcomes,
    adaptation:
      "The original explicit workspace Corsa path is absent; the public npm CLI configures its bundled Corsa. Project options use canonical vue: settings in Vite, preserving native defaults. All original tsconfig options and authored source/oracle bytes are retained.",
    configuration: {
      originalNative: {
        file: "vize.config.json",
        source: nativeConfigSource,
        sha256: digest(nativeConfigSource),
      },
      carrier: { file: "vite.config.mjs", source: configSource, sha256: digest(configSource) },
      mapping:
        "native root -> vue; lsp -> vue.languageServer; omitted native jsxTypecheck -> false",
      authority:
        "authenticated installed public authority; canonical configuration required without version inference",
    },
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
    fs.symlinkSync(options.vuePath, path.join(projectRoot, "node_modules/vue"), "dir");
    const configuration = [
      { path: path.join(projectRoot, "tsconfig.json"), source: tsconfig, sha256: digest(tsconfig) },
      {
        path: path.join(projectRoot, "vite.config.mjs"),
        source: configSource,
        sha256: digest(configSource),
      },
    ];
    for (const input of configuration) fs.writeFileSync(input.path, input.source);
    fs.writeFileSync(path.join(projectRoot, "E.vue"), testCase.input);
    assert.equal(fs.realpathSync(projectRoot), projectRoot);
    const expected = rebindFrozenExpected(testCase, {
      oldUri: testCase.authoredUri,
      newUri: pathToFileURL(path.join(projectRoot, "E.vue")).href,
    });
    // Entire independently frozen expectation is materialized before launching or querying.
    fs.writeFileSync(path.join(caseRoot, "expected.json"), JSON.stringify(expected, null, 2));
    const journalPath = path.join(caseRoot, "native-loader.ndjson");
    const outcome: Record<string, unknown> = {
      id: testCase.id,
      context: testCase.context,
      status: "running",
      projectRoot,
      source: testCase.input,
      configuration,
      expected,
      vueLink: { path: path.join(projectRoot, "node_modules/vue"), target: options.vuePath },
    };
    outcomes.push(outcome);
    persist();
    let session: InstalledAliasSession | undefined;
    try {
      recheckPublicBytes(options.authority);
      for (const input of configuration) exactFile(input.path, input.sha256, projectRoot);
      options.recheckVue();
      session = await InstalledAliasSession.launch({
        nodePath: options.authority.node.path,
        cliPath: options.authority.cli.binPath,
        custodyHookPath: options.authority.custodyHook.path,
        projectRoot,
        env: publicEnvironment(options.authority, journalPath),
        outputRoot: caseRoot,
        timeoutMs: options.timeoutMs,
        crossFile: true,
      });
      outcome.actual = await observeFrozenCase(
        session,
        testCase,
        expected,
        projectRoot,
        (actual) => {
          outcome.actual = actual;
          persist();
        },
      );
    } catch (error) {
      outcome.error =
        error instanceof Error
          ? {
              message: error.message,
              stack: error.stack,
              cause: inspect(error.cause, {
                depth: null,
                maxArrayLength: null,
                maxStringLength: null,
              }),
            }
          : String(error);
    } finally {
      if (session) outcome.terminal = await session.shutdown();
      // Initialize failures also retain their full transport directory and native journal.
      outcome.transportDirectories = fs
        .readdirSync(caseRoot)
        .filter((name) => name.startsWith("session-"))
        .map((name) => path.join(caseRoot, name));
      try {
        const terminal = outcome.terminal as { pid?: number } | undefined;
        outcome.nativeCustody = verifyNativeJournal(journalPath, options.authority, terminal?.pid);
        for (const input of configuration) exactFile(input.path, input.sha256, projectRoot);
        recheckPublicBytes(options.authority);
        options.recheckVue();
      } catch (error) {
        outcome.custodyError = String(error);
      }
      persist();
    }
    try {
      assert.equal(outcome.error, undefined);
      assert.equal(outcome.custodyError, undefined);
      const terminal = outcome.terminal as ReturnType<InstalledAliasSession["receipt"]>;
      assert.equal(terminal.closed, true);
      assert.equal(terminal.exitCode, 0);
      assert.equal(terminal.signal, null);
      assert.equal(terminal.remainingFrameBytes, 0);
      assert.deepEqual(terminal.failures, []);
      assert.deepEqual(outcome.actual, expected, "whole original authored transaction must match");
      outcome.status = "success";
    } catch (error) {
      outcome.status = "failure";
      outcome.assertion = String(error);
    }
    persist();
  }
  assert.equal(outcomes.length, 32);
  assert.ok(
    outcomes.every((outcome) => outcome.status === "success"),
    `installed original32 failed; complete receipt ${options.outputRoot}`,
  );
  return receipt;
}
