import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync } from "node:fs";
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
function processes(nonce: string): ProcessIdentity[] {
  const marker = ownerVariable + "=" + nonce;
  if (process.platform === "linux")
    return readdirSync("/proc")
      .filter((name) => /^[0-9]+$/u.test(name))
      .flatMap((name) => {
        try {
          const before = linuxIdentity(name);
          try {
            // Discard all environment bytes after testing this nonsecret owner marker.
            const environment = readFileSync("/proc/" + name + "/environ");
            before.ownerMarker = Buffer.concat([Buffer.from([0]), environment]).includes(
              Buffer.from("\0" + marker + "\0"),
            );
          } catch (value) {
            const code = (value as NodeJS.ErrnoException).code;
            if (code !== "EACCES" || statSync("/proc/" + name).uid === process.getuid?.())
              throw value;
          }
          const after = linuxIdentity(name);
          return key(before) === key(after) ? [{ ...after, ownerMarker: before.ownerMarker }] : [];
        } catch (value) {
          if (["ENOENT", "ESRCH"].includes((value as NodeJS.ErrnoException).code ?? "")) return [];
          throw value;
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
/** Identity plus a per-invocation owner marker survives setsid and immediate reparenting. */
export class OwnedProcesses {
  readonly identities = new Map<string, ProcessIdentity>();
  error: string | null = null;
  readonly rootPid: number | undefined;
  readonly nonce: string;
  constructor(rootPid: number | undefined, nonce: string) {
    this.rootPid = rootPid;
    this.nonce = nonce;
    if (rootPid) {
      try {
        const root = processes(nonce).find((row) => row.pid === rootPid);
        if (root) this.identities.set(key(root), root);
        else this.error = "root start identity was not observed";
      } catch (value) {
        this.error = String(value);
      }
    } else this.error = "root PID missing";
  }
  scan(): ProcessIdentity[] {
    try {
      const rows = processes(this.nonce);
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
      this.error = String(value);
      return [];
    }
  }
  killAll() {
    const signaled = [];
    for (const row of this.scan()) {
      try {
        // Recheck start identity and ownership immediately before touching the PID.
        const current = processes(this.nonce).find((current) => current.pid === row.pid);
        if (!current || key(current) !== key(row)) continue;
        process.kill(row.pid, "SIGKILL");
        signaled.push(row);
      } catch (value) {
        if ((value as NodeJS.ErrnoException).code !== "ESRCH") this.error = String(value);
      }
    }
    return signaled;
  }
}
