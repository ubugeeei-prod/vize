// The only prospective execution boundary; require genuine successful return.
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { CustodyError, digest, exactPath, lookup, rejectOverrides } from "./identity.ts";

export interface NativeConfiguration {
  readonly schema?: string;
  readonly installRoot: string;
  readonly nativePath: string;
  readonly nativeSha256: string;
  readonly journalPath?: string;
}
type JournalEvent = Readonly<Record<string, unknown>>;

function command(
  node: string,
  args: readonly string[],
  install: string,
  env: NodeJS.ProcessEnv,
): string {
  return execFileSync(node, args, { env, cwd: install, encoding: "utf8", timeout: 30_000 }).trim();
}

function event(value: unknown): JournalEvent {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new CustodyError("native journal must contain complete records");
  return value as JournalEvent;
}

export function verifyJournal(
  journal: string,
  configuration: NativeConfiguration,
  node: string,
  pid: number,
  corsaPath: string,
): JournalEvent {
  const data = fs.readFileSync(exactPath(journal));
  if (data.length === 0 || data.at(-1) !== 0x0a)
    throw new CustodyError("native journal must contain complete records");
  const events = data
    .toString("utf8")
    .slice(0, -1)
    .split(/\r\n|\r|\n/u)
    .map((line) => event(JSON.parse(line)));
  const first = events[0];
  if (
    first?.event !== "initialized" ||
    events.filter((item) => item.event === "initialized").length !== 1 ||
    first.nativePath !== configuration.nativePath ||
    first.sha256 !== configuration.nativeSha256 ||
    first.installRoot !== configuration.installRoot ||
    first.node !== node
  )
    throw new CustodyError("native journal initialization differs");
  let attempts = 0;
  const returned: JournalEvent[] = [];
  for (const item of events) {
    if (
      item.schema !== "vize-public-native-custody-event-v1" ||
      item.pid !== pid ||
      !["initialized", "attempt", "returned", "exit"].includes(String(item.event))
    )
      throw new CustodyError("native journal rejected/failed/unknown event or PID differs");
    if (item.expectedNative !== true) continue;
    if (item.actualPath !== configuration.nativePath || item.sha256 !== configuration.nativeSha256)
      throw new CustodyError("native journal path/digest differs");
    if (item.event === "attempt") attempts += 1;
    if (item.event === "returned") {
      if (attempts <= returned.length || item.corsaPath !== corsaPath)
        throw new CustodyError("native return lacks its attempt or bundled Corsa route");
      returned.push(item);
    }
  }
  const exits = events.filter((item) => item.event === "exit");
  const [exit] = exits;
  if (
    returned.length !== 1 ||
    attempts !== 1 ||
    exits.length > 1 ||
    (exit !== undefined && (exit.code !== 0 || events.at(-1) !== exit))
  )
    throw new CustodyError("actual native loader successful-return custody missing");
  return returned[0] ?? {};
}

export function publicProbe(
  node: string,
  hook: string,
  install: string,
  platformName: string,
  cliBin: string,
  journal: string,
  version: string,
  corsaPath: string,
): Record<string, unknown> & { node: unknown; stdout: string; stderr: string; exitCode: number } {
  rejectOverrides(process.env);
  const environment: NodeJS.ProcessEnv = { ...process.env };
  const nodeDigest = digest(fs.readFileSync(node));
  // Values have already been rejected, so no nonempty ambient override is erased.
  const information: unknown = JSON.parse(
    command(
      node,
      [
        "-e",
        "process.stdout.write(JSON.stringify({version:process.version,platform:process.platform,arch:process.arch}))",
      ],
      install,
      environment,
    ),
  );
  if (lookup(information, "platform") !== "darwin" || lookup(information, "arch") !== "arm64")
    throw new CustodyError("this collector requires the genuine Darwin ARM64 host");
  const native = exactPath(
    command(
      node,
      [
        "-e",
        "process.stdout.write(require.resolve(process.argv[1],{paths:[process.argv[2]]}))",
        platformName,
        install,
      ],
      install,
      environment,
    ),
    path.join(install, "node_modules", platformName),
  );
  if (path.extname(native) !== ".node")
    throw new CustodyError("resolved native provider must be the public platform .node file");
  const configuration = {
    schema: "vize-public-native-custody-v1",
    installRoot: install,
    nativePath: native,
    nativeSha256: digest(fs.readFileSync(native)),
    journalPath: journal,
  };
  environment.VIZE_PUBLIC_NATIVE_CUSTODY = JSON.stringify(configuration);
  const probe = spawnSync(node, ["--require", hook, cliBin, "--version"], {
    env: environment,
    cwd: install,
    encoding: "utf8",
    timeout: 30_000,
    killSignal: "SIGKILL",
  });
  if (probe.error && "code" in probe.error && probe.error.code === "ETIMEDOUT")
    throw new CustodyError("actual public CLI version probe timed out");
  if (probe.error) throw probe.error;
  if (probe.status !== 0 || probe.stdout !== `vize ${version}\n` || probe.stderr !== "")
    throw new CustodyError(
      `actual public CLI version failed: ${JSON.stringify([probe.status, probe.stdout, probe.stderr])}`,
    );
  const returned = verifyJournal(journal, configuration, node, probe.pid, corsaPath);
  if (digest(fs.readFileSync(native)) !== configuration.nativeSha256)
    throw new CustodyError("native bytes changed after genuine loader return");
  if (digest(fs.readFileSync(node)) !== nodeDigest)
    throw new CustodyError("explicit Node executable changed during collection");
  return {
    node: { path: node, version: lookup(information, "version"), sha256: nodeDigest },
    stdout: probe.stdout,
    stderr: probe.stderr,
    exitCode: probe.status,
    native: {
      packageName: platformName,
      version,
      path: native,
      sha256: configuration.nativeSha256,
      actualLoadedPath: returned.actualPath,
      journalPath: journal,
      journalSha256: digest(fs.readFileSync(journal)),
      successfulReturnObserved: true,
    },
  };
}
