import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";

import { ProcSampler } from "../proc-sampler.ts";
import type { FileIdentity, ImageIdentity, ObservationRecord, ProcReader } from "../types.ts";

export const server: ImageIdentity = Object.freeze({
  role: "server", realpath: "/synthetic/images/vize", sha256: "a".repeat(64),
  dev: "1", ino: "1", bytes: 100, mtimeNs: "10", ctimeNs: "11",
});
export const native: ImageIdentity = Object.freeze({
  role: "native", realpath: "/synthetic/images/tsc", sha256: "b".repeat(64),
  dev: "1", ino: "2", bytes: 200, mtimeNs: "12", ctimeNs: "13",
});

type Task = { birth: string; command: string; children: number[] };
type Process = {
  parent: number; birth: string; image: ImageIdentity; tasks: Map<number, Task>;
};

export function procStat(pid: number, parent: number, birth: string, command = "synthetic"): string {
  const fields = Array<string>(40).fill("0");
  fields[0] = String(parent);
  fields[10] = "11";
  fields[11] = "7";
  fields[18] = birth;
  fields[20] = "25";
  fields[35] = "3";
  return `${pid} (${command}) S ${fields.join(" ")}\n`;
}

export class ProcFixture implements ProcReader {
  readonly processes = new Map<number, Process>();
  readonly readCounts = new Map<string, number>();
  hook: ((file: string, count: number, signal: AbortSignal) => void | Promise<void>) | undefined;
  status = "VmRSS: 100 kB\nVmHWM: 120 kB\nRssAnon: 60 kB\nRssFile: 35 kB\nRssShmem: 5 kB\nVmSwap: 0 kB\n";
  rollup = "Rss: 100 kB\nPss: 70 kB\nAnonymous: 60 kB\nPss_Anon: 50 kB\nPss_File: 20 kB\n";

  constructor() {
    this.processes.set(900, {
      parent: 42, birth: "9007199254740993", image: server,
      tasks: new Map([
        [900, { birth: "9007199254740993", command: "vize", children: [910] }],
        [901, { birth: "401", command: "vize-project-navigation", children: [] }],
        [902, { birth: "402", command: "vize-project-navigation", children: [] }],
      ]),
    });
    this.addNative(910, "510");
  }

  addNative(pid: number, birth: string, parent = 900): void {
    this.processes.set(pid, {
      parent, birth, image: native,
      tasks: new Map([[pid, { birth, command: "tsc", children: [] }]]),
    });
  }

  private process(file: string): { pid: number; value: Process } {
    const match = /^\/proc\/(\d+)\//u.exec(file);
    assert.ok(match?.[1], `unexpected fixture path ${file}`);
    const pid = Number(match[1]);
    const value = this.processes.get(pid);
    assert.ok(value, `synthetic process ${pid} exited`);
    return { pid, value };
  }

  async read(file: string, _maxBytes: number, signal: AbortSignal): Promise<string> {
    signal.throwIfAborted();
    const count = (this.readCounts.get(file) ?? 0) + 1;
    this.readCounts.set(file, count);
    await this.hook?.(file, count, signal);
    signal.throwIfAborted();
    const { pid, value } = this.process(file);
    const task = /\/task\/(\d+)\/(stat|children)$/u.exec(file);
    if (task) {
      const tid = Number(task[1]);
      const entry = value.tasks.get(tid);
      assert.ok(entry, `synthetic TID ${tid} exited`);
      return task[2] === "stat"
        ? procStat(tid, value.parent, entry.birth, entry.command)
        : entry.children.join(" ");
    }
    if (file.endsWith("/stat")) return procStat(pid, value.parent, value.birth);
    if (file.endsWith("/status")) return this.status;
    if (file.endsWith("/smaps_rollup")) return this.rollup;
    throw new Error(`unimplemented synthetic proc read ${file}`);
  }

  async link(file: string, signal: AbortSignal): Promise<string> {
    signal.throwIfAborted();
    return this.process(file).value.image.realpath;
  }

  async list(file: string, signal: AbortSignal): Promise<string[]> {
    signal.throwIfAborted();
    assert.ok(file.endsWith("/task"));
    return [...this.process(file).value.tasks.keys()].map(String);
  }

  async identity(file: string, signal: AbortSignal): Promise<FileIdentity> {
    signal.throwIfAborted();
    const value = this.process(file).value.image;
    return { dev: value.dev, ino: value.ino, bytes: value.bytes, mtimeNs: value.mtimeNs, ctimeNs: value.ctimeNs };
  }
}

export async function eventually(check: () => boolean, message = "synthetic async work did not settle"): Promise<void> {
  for (let index = 0; index < 100; index += 1) {
    if (check()) return;
    await setImmediate();
  }
  assert.ok(check(), message);
}

export async function setup(fixture = new ProcFixture(), options: Partial<ConstructorParameters<typeof ProcSampler>[0]> = {}) {
  const records: ObservationRecord[] = [];
  const sampler = new ProcSampler({ rootPid: 900, images: [server, native], reader: fixture,
    emit: (record) => records.push(record), ...options });
  await eventually(() => sampler.snapshotSummary().root !== null || sampler.snapshotSummary().invalid);
  return { fixture, sampler, records };
}

export function samples(records: ObservationRecord[]) {
  return records.filter((record) => record.kind === "sample");
}

export async function close(sampler: ProcSampler): Promise<void> {
  sampler.stop("synthetic-control-cleanup");
  await eventually(() => !sampler.snapshotSummary().inFlight);
}
