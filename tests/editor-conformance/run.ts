// TS-45 runner: `node tests/editor-conformance/run.ts <neovim|helix|zed|vscode> [--out <dir>]`
//
// 1. materializes the shared `real-vue` fixture in a fresh temporary workspace;
// 2. puts a `vize` shim first on PATH that runs the real server (VIZE_SERVER_PATH
//    or target/{ci,release,debug}/vize) behind `lsp-tap.mjs`, so each editor
//    launches the server exactly the way its packaged integration does;
// 3. lets the client's driver play the scenario through the editor;
// 4. judges the transcript with `conformance.ts`, then re-judges it with every
//    step's expectation corrupted (each corrupted step must fail);
// 5. writes `<out>/<client>.json` for `report.ts` and prints this client's column.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  prepareRealVueWorkspace,
  resolveRealServerPath,
} from "../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { evaluate, WORKSPACE } from "./conformance.ts";
import { negativeControl } from "./negative-control.ts";
import { loadScenario, suiteRoot, type Driver, type DriverContext } from "./support/context.ts";
import { readTranscript, waitForTranscript } from "./support/transcript.ts";
import { renderTable, type ClientResult } from "./report.ts";

const drivers: Record<string, () => Promise<{ driver: Driver }>> = {
  helix: () => import("./drivers/helix.ts"),
  neovim: () => import("./drivers/neovim.ts"),
  vscode: () => import("./drivers/vscode.ts"),
  zed: () => import("./drivers/zed.ts"),
};

async function main(argv: string[]): Promise<number> {
  const id = argv[0];
  const load = drivers[id];
  if (load == null)
    throw new Error(`usage: run.ts <${Object.keys(drivers).join("|")}> [--out <dir>]`);
  const outFlag = argv.indexOf("--out");
  const out = path.resolve(outFlag >= 0 ? argv[outFlag + 1] : path.join(os.tmpdir(), "vize-ts45"));
  fs.mkdirSync(out, { recursive: true });

  const { driver } = await load();
  const scenario = loadScenario();
  const session = fs.mkdtempSync(path.join(os.tmpdir(), `vize-ts45-${id}-`));
  const workspace = prepareRealVueWorkspace(path.join(session, "real-vue")) as string;
  const shimDir = path.join(session, "bin");
  fs.mkdirSync(shimDir);
  const server = resolveRealServerPath() as string;
  const tap = path.join(suiteRoot, "lsp-tap.mjs");
  fs.writeFileSync(
    path.join(shimDir, "vize"),
    `#!/bin/sh\nexec ${JSON.stringify(process.execPath)} ${JSON.stringify(tap)} ${JSON.stringify(server)} "$@"\n`,
    { mode: 0o755 },
  );
  const transcript = path.join(out, `${id}.transcript.jsonl`);
  fs.rmSync(transcript, { force: true });
  const context: DriverContext = {
    scenario,
    workspace,
    roots: [...new Set([workspace, fs.realpathSync(workspace)])],
    documentPath: path.join(workspace, scenario.document),
    documentKey: `${WORKSPACE}/${scenario.document}`,
    env: {
      ...process.env,
      PATH: `${shimDir}${path.delimiter}${process.env.PATH ?? ""}`,
      VIZE_CONFORMANCE_TRANSCRIPT: transcript,
    },
    transcript,
    scratch: path.join(session, "editor"),
  };
  fs.mkdirSync(context.scratch);

  const startedAt = Date.now();
  let driverError: string | null = null;
  try {
    await driver.run(context);
  } catch (error) {
    driverError = error instanceof Error ? (error.stack ?? error.message) : String(error);
    console.error(`[ts-45] ${driver.client} driver failed: ${driverError}`);
  }
  // Every server the editor started must have exited (the tap's exit
  // recorder appends that record, possibly after the editor itself is gone).
  await waitForTranscript(
    transcript,
    "every server session to exit",
    (seen) => {
      const spawned = seen.filter((entry) => entry.event === "spawn").map((entry) => entry.session);
      const exited = new Set(
        seen.filter((entry) => entry.event === "exit").map((entry) => entry.session),
      );
      return spawned.every((session) => exited.has(session)) ? true : undefined;
    },
    30_000,
  ).catch((error: unknown) => console.error(`[ts-45] ${String(error)}`));
  const entries = readTranscript(transcript);
  const evaluation = evaluate(scenario, entries, context.roots);
  const result: ClientResult = {
    id,
    client: driver.client,
    version: driver.version(),
    mode: driver.mode,
    seconds: Math.round((Date.now() - startedAt) / 1000),
    driverError,
    evaluation,
    corruptedExpectationsAccepted: negativeControl(scenario, entries, context.roots),
  };
  fs.writeFileSync(path.join(out, `${id}.json`), `${JSON.stringify(result, null, 2)}\n`);
  if (process.env.VIZE_TS45_KEEP_WORKSPACE == null)
    fs.rmSync(session, { force: true, recursive: true });

  console.log(renderTable(scenario, [result]));
  for (const step of evaluation.steps.filter((candidate) => !candidate.pass)) {
    console.error(`[ts-45] ${driver.client} ${step.id}: ${step.detail}`);
  }
  const passed =
    driverError == null &&
    evaluation.steps.every((candidate) => candidate.pass) &&
    result.corruptedExpectationsAccepted.length === 0;
  return passed ? 0 : 1;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = await main(process.argv.slice(2));
}
