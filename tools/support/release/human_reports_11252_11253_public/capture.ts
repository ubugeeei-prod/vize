import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { CommandPlan, InstallReceipt, NativeEvent, Observation } from "./model.ts";
import { prohibited } from "./processes.ts";
import { bytes, object, sha256, sourcePath, text } from "./value.ts";
import { discard, type Sink } from "./evidence.ts";
import { inputWitness } from "./witness.ts";
import { requireAmbientBinConfiguration } from "./bin-configuration.ts";
export function cleanEnvironment(
  sink: Sink = discard,
  ambient: NodeJS.ProcessEnv = process.env,
): NodeJS.ProcessEnv {
  requireAmbientBinConfiguration(ambient, sink);
  for (const key of prohibited)
    assert.equal(ambient[key] ?? "", "", `initial source/provider override: ${key}`);
  assert.equal(ambient.VIZE_HUMAN_REPORT_NATIVE_OBSERVER ?? "", "");
  const env = { ...ambient };
  for (const key of ["GITHUB_TOKEN", "GH_TOKEN", "NODE_AUTH_TOKEN", "NPM_TOKEN"]) delete env[key];
  return env;
}
export function capture(
  command: string,
  argv: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  output: string,
  id: string,
): {
  pid: number;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdoutBase64: string;
  stderrBase64: string;
} {
  assert.match(id, /^[a-z][a-z0-9-]*$/u);
  const result = spawnSync(command, argv, {
    cwd,
    env,
    timeout: 600_000,
    maxBuffer: 32 * 1024 * 1024,
    windowsHide: true,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  writeFileSync(join(output, `${id}.stdout.txt`), stdout, { flag: "wx" });
  writeFileSync(join(output, `${id}.stderr.txt`), stderr, { flag: "wx" });
  const row = {
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error ? String(result.error) : null,
    stdoutBase64: stdout.toString("base64"),
    stderrBase64: stderr.toString("base64"),
  };
  writeFileSync(
    join(output, `${id}.process.json`),
    JSON.stringify(
      { ...row, command, argv, cwd, stdoutSha256: sha256(stdout), stderrSha256: sha256(stderr) },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  return row;
}
export function writeInputs(plan: CommandPlan, root: string): void {
  for (const file of plan.sourceFiles) {
    sourcePath(file.path);
    assert(
      file.path === "package.json" ||
        file.path === "tsconfig.json" ||
        /^src\/(?:[\w.-]+\/)*[\w.-]+\.(?:vue|ts)$/u.test(file.path),
      "finite recipe input path required",
    );
    const raw = bytes(file.base64);
    assert.equal(sha256(raw), file.sha256);
    const target = join(root, file.path);
    mkdirSync(join(target, ".."), { recursive: true });
    writeFileSync(target, raw);
  }
}
export function recordCommand(
  plan: CommandPlan,
  install: InstallReceipt,
  output: string,
  companion: boolean,
  sink: Sink = discard,
  inputAlreadyPrepared = false,
): Observation {
  const env = { ...cleanEnvironment(sink), ...install.packageManagerEnvironment },
    journalRoot = join(output, `${plan.id}-native-journal`);
  if (companion) {
    mkdirSync(journalRoot);
    const observer = realpathSync(fileURLToPath(new URL("./native-observer.ts", import.meta.url)));
    // This is the complete named instrumentation delta. No mapper exec,
    // package, checker-selection variable or installed file is replaced.
    env.NODE_OPTIONS = `--import=${pathToFileURL(observer).href}`;
    env.VIZE_HUMAN_REPORT_NATIVE_OBSERVER = JSON.stringify({
      schema: "vize.humanReportNativeObserver.v1",
      nativePath: install.native.path,
      nativeSha256: install.native.sha256,
      journalRoot,
      commandId: plan.id,
      rootPid: process.pid,
    });
  }
  if (!inputAlreadyPrepared) writeInputs(plan, install.root);
  sink("actual-input-before-process", inputWitness(plan, install.root));
  const id = plan.id + (companion ? "-companion" : "-original");
  const result = capture(plan.executable, plan.argv, install.root, env, output, id);
  const nativeEvents: NativeEvent[] = [];
  if (companion)
    for (const name of readdirSync(journalRoot).sort()) {
      assert.match(name, /^[1-9]\d*\.jsonl$/u);
      const raw = readFileSync(join(journalRoot, name));
      for (const line of raw.toString("utf8").split("\n").filter(Boolean)) {
        const packet = object(JSON.parse(line));
        text(packet.event);
        nativeEvents.push(packet as unknown as NativeEvent);
      }
    }
  const recordedEnvironment = Object.fromEntries(prohibited.map((key) => [key, env[key] ?? ""]));
  Object.assign(recordedEnvironment, install.packageManagerEnvironment);
  recordedEnvironment.VIZE_HUMAN_REPORT_NATIVE_OBSERVER =
    env.VIZE_HUMAN_REPORT_NATIVE_OBSERVER ?? "";
  return {
    id: plan.id,
    consumer: plan.consumer,
    ...result,
    executable: plan.executable,
    argv: plan.argv,
    cwd: install.root,
    sourceFiles: plan.sourceFiles,
    checker: plan.checker,
    environment: recordedEnvironment,
    nativeEvents,
    instrumentation: companion ? "load-observed-companion" : "none",
    ...(companion ? { companionOf: plan.id } : {}),
    backend: "UNKNOWN",
  };
}
