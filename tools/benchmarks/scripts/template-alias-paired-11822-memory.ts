import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import {
  closeSync,
  openSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { childEnvironment, evidence } from "./template-alias-paired-11822-custody.ts";

export function parseStat(text: string) {
  const boundary = text.lastIndexOf(")");
  assert.ok(boundary > 0);
  const pid = Number(text.slice(0, text.indexOf(" ")));
  const fields = text
    .slice(boundary + 2)
    .trim()
    .split(/\s+/u);
  assert.ok(Number.isSafeInteger(pid) && pid > 0 && fields.length >= 20);
  assert.match(fields[19], /^[0-9]+$/u);
  return { pid, state: fields[0], ppid: Number(fields[1]), startTicks: fields[19] };
}

export function parseRollup(text: string) {
  const field = (name: string) => {
    const values = [...text.matchAll(new RegExp(`^${name}:\\s+([0-9]+) kB$`, "gm"))];
    assert.equal(values.length, 1, `missing or duplicate ${name}`);
    const value = Number(values[0][1]);
    assert.ok(Number.isSafeInteger(value) && value >= 0);
    return value;
  };
  return { rssKiB: field("Rss"), pssKiB: field("Pss") };
}

type Identity = ReturnType<typeof parseStat>;
type Role = "mapper" | "checker" | "launcher";
type Row = Identity & ReturnType<typeof parseRollup> & { role: Role; exe: string };
const identity = (pid: number) => parseStat(readFileSync(`/proc/${pid}/stat`, "utf8"));
const key = (row: Identity) => `${row.pid}:${row.startTicks}`;
const gone = (error: unknown) =>
  ["ENOENT", "ESRCH"].includes((error as NodeJS.ErrnoException).code ?? "");

/** Separate memory observations: never attach this sampler to a primary timing call. */
export async function observeMemory(options: {
  command: string[];
  cwd: string;
  output: string;
  hostImage: string;
  mapperImage: string;
  timeoutMilliseconds: number;
  environment?: NodeJS.ProcessEnv;
}) {
  assert.equal(process.platform, "linux");
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.ok(options.timeoutMilliseconds > 0 && options.timeoutMilliseconds <= 300_000);
  const expectedHost = evidence(options.hostImage),
    expectedMapper = evidence(options.mapperImage);
  const stdout = openSync(options.output + ".stdout", "wx"),
    stderr = openSync(options.output + ".stderr", "wx");
  const child = spawn(options.command[0], options.command.slice(1), {
    cwd: options.cwd,
    env: childEnvironment({ ...options.environment, NO_COLOR: "1" }),
    stdio: ["ignore", stdout, stderr],
    detached: true,
  });
  child.on("error", () => {});
  let root: Identity;
  try {
    assert.ok(child.pid);
    root = identity(child.pid);
  } catch (error) {
    child.kill("SIGKILL");
    child.unref();
    closeSync(stdout);
    closeSync(stderr);
    writeFileSync(
      options.output + ".json",
      JSON.stringify({ timingEligible: false, rootIdentityError: String(error) }) + "\n",
      { flag: "wx" },
    );
    throw error;
  }
  const known = new Map<string, Identity>([[key(root), root]]);
  const images = new Map<string, ReturnType<typeof evidence>>();
  const samples: {
    beganMilliseconds: number;
    endedMilliseconds: number;
    rows: Row[];
    rssKiB: number;
    pssKiB: number;
  }[] = [];
  const errors: string[] = [];
  const origin = performance.now();
  let timedOut = false,
    status: number | null = null,
    signal: NodeJS.Signals | null = null,
    closeObserved = false;
  const observedRoles = new Set<Role>();
  const inspect = (pid: number, rows: Row[], visited: Set<number>, parent?: number) => {
    if (visited.has(pid)) return;
    visited.add(pid);
    try {
      const before = identity(pid);
      if (before.state === "Z") return;
      const old = [...known.values()].find((row) => row.pid === pid);
      if (old && key(old) !== key(before)) return;
      if (!old && parent !== undefined && before.ppid !== parent) return;
      known.set(key(before), before);
      const children = new Set<number>();
      for (const thread of readdirSync(`/proc/${pid}/task`)) {
        try {
          for (const name of readFileSync(`/proc/${pid}/task/${thread}/children`, "utf8")
            .trim()
            .split(/\s+/u)
            .filter(Boolean))
            children.add(Number(name));
        } catch (error) {
          if (!gone(error)) throw error;
        }
      }
      const exe = readlinkSync(`/proc/${pid}/exe`);
      const maps = readFileSync(`/proc/${pid}/maps`, "utf8");
      const memory = parseRollup(readFileSync(`/proc/${pid}/smaps_rollup`, "utf8"));
      const after = identity(pid);
      if (key(before) !== key(after)) return;
      const role: Role =
        exe === expectedHost.path
          ? "checker"
          : maps.split("\n").some((line) => line.endsWith(" " + expectedMapper.path))
            ? "mapper"
            : "launcher";
      if (role !== "launcher") {
        const path = role === "checker" ? expectedHost.path : expectedMapper.path;
        if (!images.has(path)) images.set(path, evidence(path));
        assert.deepEqual(images.get(path), role === "checker" ? expectedHost : expectedMapper);
      }
      rows.push({ ...after, ...memory, role, exe });
      observedRoles.add(role);
      for (const descendant of children) inspect(descendant, rows, visited, before.pid);
    } catch (error) {
      if (!gone(error)) errors.push(`pid=${pid} ${String(error)}`);
    }
  };
  const sample = () => {
    const beganMilliseconds = performance.now() - origin;
    const rows: Row[] = [],
      visited = new Set<number>();
    // Retain already observed identities even when the root exits or reparents a child.
    for (const row of known.values()) inspect(row.pid, rows, visited);
    samples.push({
      beganMilliseconds,
      endedMilliseconds: performance.now() - origin,
      rows,
      rssKiB: rows.reduce((sum, row) => sum + row.rssKiB, 0),
      pssKiB: rows.reduce((sum, row) => sum + row.pssKiB, 0),
    });
  };
  const kill = () => {
    for (const row of known.values())
      try {
        if (key(identity(row.pid)) === key(row)) process.kill(row.pid, "SIGKILL");
      } catch (error) {
        if (!gone(error)) errors.push(String(error));
      }
  };
  sample();
  const sampler = setInterval(sample, 25);
  const deadline = setTimeout(() => {
    timedOut = true;
    kill();
  }, options.timeoutMilliseconds);
  let fallback: ReturnType<typeof setTimeout> | undefined;
  try {
    await new Promise<void>((done) => {
      child.once("error", (error) => {
        errors.push(String(error));
        done();
      });
      child.once("close", (code, reason) => {
        closeObserved = true;
        status = code;
        signal = reason;
        done();
      });
      fallback = setTimeout(() => {
        timedOut = true;
        kill();
        child.unref();
        done();
      }, options.timeoutMilliseconds + 5000);
    });
  } finally {
    clearInterval(sampler);
    clearTimeout(deadline);
    clearTimeout(fallback);
    sample();
    closeSync(stdout);
    closeSync(stderr);
  }
  const survivors = [...known.values()].filter((row) => {
    try {
      const current = identity(row.pid);
      return key(row) === key(current) && current.state !== "Z";
    } catch (error) {
      if (!gone(error)) errors.push(String(error));
      return false;
    }
  });
  if (survivors.length) kill();
  const receipt = {
    timingEligible: false,
    requestedIntervalMilliseconds: 25,
    scope:
      "sampled observed descendants; short-lived processes between samples can be missed; RSS/PSS observed peaks are lower bounds, not continuous maxima",
    root,
    known: [...known.values()],
    observedImages: [...images.values()],
    status,
    signal,
    timedOut,
    closeObserved,
    survivors,
    errors,
    samples,
    maximumObservedTreeRssKiB: Math.max(0, ...samples.map((row) => row.rssKiB)),
    maximumObservedTreePssKiB: Math.max(0, ...samples.map((row) => row.pssKiB)),
    roles: ["mapper", "checker", "launcher"].map((role) => ({
      role,
      maximumObservedRssKiB: Math.max(
        0,
        ...samples.map((sample) =>
          sample.rows.filter((row) => row.role === role).reduce((sum, row) => sum + row.rssKiB, 0),
        ),
      ),
    })),
  };
  writeFileSync(options.output + ".json", JSON.stringify(receipt, null, 2) + "\n", { flag: "wx" });
  assert.equal(timedOut, false);
  assert.equal(closeObserved, true);
  assert.equal(signal, null);
  assert.deepEqual(errors, []);
  assert.deepEqual(survivors, []);
  assert.ok(
    observedRoles.has("mapper") && observedRoles.has("checker"),
    "memory observation missed a required role",
  );
  return receipt;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  throw new Error("import observeMemory from the public campaign controller");
