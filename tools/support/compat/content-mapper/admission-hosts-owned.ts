import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
export const ownerVariable = "ADMISSION_PROCESS_OWNER";
export type ProcessIdentity = {
  pid: number;
  ppid: number;
  group: number;
  startTicks: string;
  state: string;
  ownerMarker: boolean;
};
const key = (row: ProcessIdentity) => row.pid + ":" + row.startTicks;
export function observationDisposition(
  identity: ProcessIdentity | undefined,
  baseline: ReadonlySet<string>,
  owned: ReadonlySet<string>,
  code: string | undefined,
) {
  const scope = !identity
    ? "identity-unverified"
    : identity.ownerMarker || owned.has(key(identity))
      ? "known-owned"
      : baseline.has(key(identity))
        ? "preexisting-unrelated"
        : "ownership-unclassified";
  return {
    scope,
    excluded: scope === "preexisting-unrelated" && ["EACCES", "EPERM"].includes(code ?? ""),
  };
}
export function linuxIdentity(name: string) {
  const stat = readFileSync("/proc/" + name + "/stat", "utf8");
  const fields = stat
    .slice(stat.lastIndexOf(")") + 2)
    .trim()
    .split(/\s+/u);
  assert.ok(fields.length >= 20);
  return {
    pid: Number(name),
    ppid: Number(fields[1]),
    group: Number(fields[2]),
    startTicks: fields[19],
    state: fields[0],
    ownerMarker: false,
  };
}
function processes(
  nonce: string,
  unknown: (error: unknown, pid?: number, identity?: ProcessIdentity) => void,
): ProcessIdentity[] {
  const marker = ownerVariable + "=" + nonce;
  if (process.platform === "linux")
    return readdirSync("/proc")
      .filter((name) => /^[0-9]+$/u.test(name))
      .flatMap((name) => {
        let before: ProcessIdentity | undefined;
        try {
          before = linuxIdentity(name);
          // Discard all environment bytes after testing this nonsecret owner marker.
          const environment = readFileSync("/proc/" + name + "/environ");
          before.ownerMarker = Buffer.concat([Buffer.from([0]), environment]).includes(
            Buffer.from("\0" + marker + "\0"),
          );
          const after = linuxIdentity(name);
          return key(before) === key(after) ? [{ ...after, ownerMarker: before.ownerMarker }] : [];
        } catch (value) {
          if (["ENOENT", "ESRCH"].includes((value as NodeJS.ErrnoException).code ?? "")) return [];
          let stable: ProcessIdentity | undefined;
          try {
            const after = linuxIdentity(name);
            if (before && key(before) === key(after))
              stable = { ...after, ownerMarker: before.ownerMarker };
          } catch (identityError) {
            if (!["ENOENT", "ESRCH"].includes((identityError as NodeJS.ErrnoException).code ?? ""))
              unknown(identityError, Number(name));
          }
          unknown(value, Number(name), stable);
          return [];
        }
      });
  assert.equal(process.platform, "darwin", "unsupported process ownership observer");
  // Darwin is used only by finite Node controls; Linux records kernel start ticks.
  const snapshot = execFileSync(
    "/bin/ps",
    ["eww", "-axo", "pid=,ppid=,pgid=,stat=,lstart=,command="],
    { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  return snapshot
    .trim()
    .split("\n")
    .map((line) => {
      const match =
        /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+([A-Z][a-z]{2}\s+[A-Z][a-z]{2}\s+\d+\s+\d\d:\d\d:\d\d\s+\d{4})\s+(.*)$/u.exec(
          line,
        );
      assert.ok(match, "unrecognized process identity row");
      return {
        pid: Number(match[1]),
        ppid: Number(match[2]),
        group: Number(match[3]),
        state: match[4],
        startTicks: "darwin-start-time:" + match[5],
        ownerMarker: (" " + match[6] + " ").includes(" " + marker + " "),
      };
    });
}
/** Capture identities before spawning; non-exit observation failures abort the invocation. */
export function captureProcessBaseline(): ProcessIdentity[] {
  if (process.platform !== "linux")
    return processes("", (value) => {
      throw value;
    });
  return readdirSync("/proc")
    .filter((name) => /^[0-9]+$/u.test(name))
    .flatMap((name) => {
      try {
        return [linuxIdentity(name)];
      } catch (value) {
        if (["ENOENT", "ESRCH"].includes((value as NodeJS.ErrnoException).code ?? "")) return [];
        throw value;
      }
    });
}
/** Identity plus a per-invocation owner marker survives setsid and immediate reparenting. */
export class OwnedProcesses {
  readonly identities = new Map<string, ProcessIdentity>();
  readonly observationIssues = new Map<
    string,
    {
      pid: number | null;
      startTicks: string | null;
      scope: string;
      code: string | null;
      excluded: boolean;
      error: string;
    }
  >();
  error: string | null = null;
  readonly rootPid: number | undefined;
  readonly nonce: string;
  readonly baseline: ReadonlySet<string>;
  recordIssue(value: unknown, pid?: number, processIdentity?: ProcessIdentity) {
    const { scope, excluded } = observationDisposition(
      processIdentity,
      this.baseline,
      new Set(this.identities.keys()),
      (value as NodeJS.ErrnoException).code,
    );
    const issue = {
      pid: pid ?? null,
      startTicks: processIdentity?.startTicks ?? null,
      scope,
      excluded,
      code: (value as NodeJS.ErrnoException).code ?? null,
      error: String(value),
    };
    const identity = JSON.stringify(issue);
    if (this.observationIssues.size < 4096 || this.observationIssues.has(identity))
      this.observationIssues.set(identity, issue);
    if (!excluded) this.error ??= "process inventory incomplete (" + scope + "): " + String(value);
  }
  constructor(
    rootPid: number | undefined,
    nonce: string,
    baseline: readonly ProcessIdentity[] = [],
  ) {
    this.rootPid = rootPid;
    this.nonce = nonce;
    this.baseline = new Set(baseline.map(key));
    if (rootPid) {
      try {
        const root =
          process.platform === "linux"
            ? linuxIdentity(String(rootPid))
            : processes(nonce, (value, pid, identity) =>
                this.recordIssue(value, pid, identity),
              ).find((row) => row.pid === rootPid);
        if (root) this.identities.set(key(root), root);
        else this.error = "root start identity was not observed";
      } catch (value) {
        this.error = String(value);
      }
    } else this.error = "root PID missing";
  }
  scan(): ProcessIdentity[] {
    try {
      const rows = processes(this.nonce, (value, pid, identity) =>
        this.recordIssue(value, pid, identity),
      );
      for (const row of rows) if (row.ownerMarker) this.identities.set(key(row), row);
      let changed = true;
      while (changed) {
        changed = false;
        const owners = new Set(
          rows.filter((row) => this.identities.has(key(row))).map((row) => row.pid),
        );
        for (const row of rows)
          if (!this.identities.has(key(row)) && owners.has(row.ppid)) {
            this.identities.set(key(row), row);
            changed = true;
          }
      }
      return rows.filter((row) => this.identities.has(key(row)));
    } catch (value) {
      this.recordIssue(value);
      return [];
    }
  }
  killAll() {
    const signaled = [];
    this.scan();
    for (const row of this.identities.values()) {
      try {
        // Recheck start identity and ownership immediately before touching the PID.
        const current =
          process.platform === "linux"
            ? linuxIdentity(String(row.pid))
            : processes(this.nonce, (value, pid, identity) =>
                this.recordIssue(value, pid, identity),
              ).find((current) => current.pid === row.pid);
        if (!current || key(current) !== key(row)) continue;
        process.kill(row.pid, "SIGKILL");
        signaled.push(row);
      } catch (value) {
        if (!["ENOENT", "ESRCH"].includes((value as NodeJS.ErrnoException).code ?? ""))
          this.error = String(value);
      }
    }
    return signaled;
  }
}
