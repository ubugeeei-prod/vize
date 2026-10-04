import { Buffer } from "node:buffer";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import type { TestContext } from "node:test";

import type { PinnedFixtureWorkspace } from "../../../_helpers/realworld-patch.ts";
import type { CommandResult, VizeCheckResult } from "../../../_helpers/realworld-typecheck.ts";
import { ownError, rawCommand, type RecordObservation } from "./raw-process.ts";
import {
  assertCli,
  configure,
  type Config,
  type ErrorSpec,
  PAGE_PATH,
  PLAYGROUND,
  ROUTES_PATH,
  sha256,
} from "./support.ts";

const configPaths = [
  "tsconfig.json",
  "vize.config.json",
  `${PLAYGROUND}/compiler-options.json`,
  "project-root-map.d.ts",
];

function readInputs(fixture: PinnedFixtureWorkspace, sourcePath: string) {
  const paths = [...new Set([sourcePath, PAGE_PATH, ROUTES_PATH, ...configPaths])];
  return new Map(
    paths.map(
      (file) =>
        [
          file,
          fs.existsSync(fixture.resolve(file)) ? fs.readFileSync(fixture.resolve(file)) : null,
        ] as const,
    ),
  );
}

function inputPacket(inputs: Map<string, Uint8Array | null>) {
  return Object.fromEntries(
    [...inputs].map(([file, bytes]) => [
      file,
      bytes === null
        ? null
        : {
            bytes: bytes.length,
            sha256: createHash("sha256").update(bytes).digest("hex"),
            base64: Buffer.from(bytes).toString("base64"),
          },
    ]),
  );
}

function restoreReferenceInputs(
  fixture: PinnedFixtureWorkspace,
  sourcePath: string,
  inputs: Map<string, Uint8Array | null>,
) {
  for (const file of [sourcePath, ...configPaths]) {
    const bytes = inputs.get(file);
    assert.notEqual(bytes, undefined, `recorded authored input: ${file}`);
    if (bytes === null) fs.rmSync(fixture.resolve(file), { force: true });
    else fs.writeFileSync(fixture.resolve(file), bytes!);
  }
}

/** Raw primary observations retain the official provider's original page/map authority. */
export function pageRouteObserver(
  t: TestContext,
  fixture: PinnedFixtureWorkspace,
  record: RecordObservation,
  cli: string,
  corsaPath: string,
  vueTscPath: string,
  originalPage: string,
  originalGeneratedRoutes: string,
) {
  const authority = {
    originalPage: { path: PAGE_PATH, source: originalPage, sha256: sha256(originalPage) },
    originalGeneratedRoutes: {
      path: ROUTES_PATH,
      source: originalGeneratedRoutes,
      sha256: sha256(originalGeneratedRoutes),
    },
  };
  return function observe(
    source: string,
    errors: ErrorSpec[],
    sourcePath = PAGE_PATH,
    reference: (source: string) => string = (value) => value,
    referenceConfig?: Config,
    productConfig: Config = {},
  ) {
    fixture.write(sourcePath, source);
    const productTsconfig = fixture.read("tsconfig.json");
    const productInputs = readInputs(fixture, sourcePath);
    const context = {
      sourcePath,
      source,
      productTsconfig,
      productConfig,
      referenceConfig,
      authority,
    };
    record({ ...context, stage: "product-input", inputs: inputPacket(productInputs) });
    let vize: VizeCheckResult;
    try {
      const result = rawCommand(
        cli,
        [
          "check",
          sourcePath,
          "--tsconfig",
          "tsconfig.json",
          "--format",
          "json",
          "--quiet",
          "--corsa-path",
          corsaPath,
        ],
        fixture.workspaceDir,
        record,
        { ...context, stage: "product-raw", inputs: inputPacket(productInputs) },
      );
      vize = { ...result, report: JSON.parse(result.stdout) as VizeCheckResult["report"] };
    } catch (error) {
      record({ ...context, stage: "product-failure", error: ownError(error) });
      throw error;
    }
    record({ stage: "product", sourcePath, source, productTsconfig, vize });
    let oracle: CommandResult;
    let oracleTsconfig: string;
    let oracleSource: string;
    try {
      record({ ...context, stage: "reference-input", inputs: inputPacket(productInputs) });
      fixture.write(sourcePath, reference(source));
      if (referenceConfig) configure(fixture, corsaPath, sourcePath, referenceConfig);
      oracleSource = fixture.read(sourcePath);
      oracleTsconfig = fixture.read("tsconfig.json");
      const referenceInputs = readInputs(fixture, sourcePath);
      const referenceContext = { ...context, oracleSource, oracleTsconfig };
      record({
        ...referenceContext,
        stage: "reference-prepared",
        inputs: inputPacket(referenceInputs),
      });
      oracle = rawCommand(
        vueTscPath,
        ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
        fixture.workspaceDir,
        record,
        { ...referenceContext, stage: "reference-raw", inputs: inputPacket(referenceInputs) },
      );
    } catch (error) {
      record({
        ...context,
        stage: "reference-failure",
        inputs: inputPacket(readInputs(fixture, sourcePath)),
        error: ownError(error),
      });
      throw error;
    } finally {
      restoreReferenceInputs(fixture, sourcePath, productInputs);
    }
    record({ sourcePath, source, productTsconfig, oracleSource, oracleTsconfig, vize, oracle });
    t.diagnostic(JSON.stringify({ sourcePath, sourceSha256: sha256(source), vize, oracle }));
    const include = (JSON.parse(productTsconfig) as { include?: unknown }).include;
    assert.ok(Array.isArray(include) && include.every((file) => typeof file === "string"));
    const rows = assertCli(vize, oracle, source, errors, sourcePath, include);
    return { rows, report: vize.report };
  };
}
