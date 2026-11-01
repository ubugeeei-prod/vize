import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { mock, test } from "node:test";
import {
  hashMeasurements,
  observeDescendants,
} from "../../.github/actions/test-js-packages-with-history/native-fault-support.ts";

type Image = { path: string; sha256: string; fileIdentity: string };
type Sample = {
  pid: number;
  parentPid: number;
  parentStartTicks: string | null;
  startTicks: string;
  executable: string;
  executableIdentity: Image;
  selection: string;
  tasks: { tid: number; tgid: number; startTicks: string }[];
  unavailable: number[];
  limited: boolean;
};
type Summary = { limited: boolean; gaps: { pid: number; identity: string }[] };

function stat(pid: number, parent: number, start: string): string {
  const tail = Array<string>(20).fill("0");
  tail[0] = "S";
  tail[1] = String(parent);
  tail[19] = start;
  return `${pid} (actor with spaces) ${tail.join(" ")}\n`;
}

function fixture() {
  const root = fs.mkdtempSync(path.join(tmpdir(), "vize-process-identity-"));
  const proc = path.join(root, "proc");
  const output = path.join(root, "output");
  fs.mkdirSync(proc);
  const actor = (pid: number, parent: number, start: string, filename: string) => {
    const directory = path.join(proc, String(pid));
    fs.mkdirSync(directory);
    fs.writeFileSync(path.join(directory, "stat"), stat(pid, parent, start));
    fs.symlinkSync(filename, path.join(directory, "exe"));
    fs.symlinkSync(root, path.join(directory, "cwd"));
    fs.writeFileSync(
      path.join(directory, "maps"),
      `1000-2000 r-xp 0 00:00 ${fs.statSync(filename).ino} ${filename}\n`,
    );
    fs.mkdirSync(path.join(directory, "task"));
    return directory;
  };
  const task = (pid: number, tid: number, start: string, group = pid) => {
    const directory = path.join(proc, String(pid), "task", String(tid));
    fs.mkdirSync(directory);
    fs.writeFileSync(path.join(directory, "stat"), stat(tid, 0, start));
    fs.writeFileSync(path.join(directory, "status"), `Tgid:\t${group}\nPid:\t${tid}\n`);
  };
  const samples = (): Sample[] =>
    fs
      .readdirSync(path.join(output, "processes"))
      .map(
        (name) =>
          JSON.parse(fs.readFileSync(path.join(output, "processes", name), "utf8")) as Sample,
      );
  const summary = (): Summary =>
    JSON.parse(fs.readFileSync(path.join(output, "process-sampling.json"), "utf8")) as Summary;
  const image = (name: string) => {
    const filename = path.join(root, name);
    fs.writeFileSync(filename, `physical ${name}`);
    return filename;
  };
  return { root, proc, output, actor, task, samples, summary, image };
}

const unix = { skip: process.platform === "win32" };

void test(
  "non-Node descendant PPID/start/TID joins retain the physical executable and cached image",
  unix,
  () => {
    const f = fixture();
    try {
      const executable = f.image("native-tool");
      f.actor(100, 1, "111", executable);
      f.task(100, 100, "111");
      f.actor(101, 100, "222", executable);
      f.task(101, 101, "222");
      f.task(101, 8869, "333");
      const seen = new Set<string>();
      const images = new Map();
      observeDescendants(100, seen, images, f.output, f.proc);
      const child = f.samples().find((item) => item.pid === 101)!;
      assert.equal(child.parentPid, 100);
      assert.equal(child.parentStartTicks, "111");
      assert.equal(child.startTicks, "222");
      assert.deepEqual(child.tasks, [
        { tid: 101, tgid: 101, startTicks: "222" },
        { tid: 8869, tgid: 101, startTicks: "333" },
      ]);
      assert.equal(child.executableIdentity.path, executable);
      assert.equal(
        child.executableIdentity.sha256,
        createHash("sha256").update(fs.readFileSync(executable)).digest("hex"),
      );
      assert.match(child.selection, /partial/);
      assert.equal(images.size, 1);
      const bytes = hashMeasurements.bytes;
      observeDescendants(100, seen, images, f.output, f.proc);
      assert.equal(hashMeasurements.bytes, bytes);
      assert.equal(f.samples().length, 2);
      fs.writeFileSync(executable, "a changed physical image");
      observeDescendants(100, seen, images, f.output, f.proc);
      assert.equal(images.size, 2);
      assert.equal(f.samples().length, 4);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  },
);

void test(
  "PID reuse during collection refuses a joined actor instead of retaining stale identity",
  unix,
  () => {
    const f = fixture();
    const originalOpen = fs.openSync;
    try {
      const executable = f.image("native-tool");
      const actor = f.actor(100, 1, "111", executable);
      f.task(100, 100, "111");
      let reads = 0;
      const interception = mock.method(
        fs,
        "openSync",
        (filename: fs.PathLike, flags: fs.OpenMode, mode?: fs.Mode) => {
          if (filename === path.join(actor, "stat") && ++reads === 2)
            fs.writeFileSync(filename, stat(100, 1, "999"));
          return originalOpen(filename, flags, mode);
        },
      );
      try {
        observeDescendants(100, new Set(), new Map(), f.output, f.proc);
      } finally {
        interception.mock.restore();
      }
      assert.deepEqual(f.summary().gaps, [{ pid: 100, identity: "UNKNOWN", unavailable: "Error" }]);
      assert.equal(fs.existsSync(path.join(f.output, "processes")), false);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  },
);

void test(
  "foreign task groups and oversized proc text remain unknown; task census is bounded",
  unix,
  () => {
    const f = fixture();
    try {
      const executable = f.image("native-tool");
      const actor = f.actor(100, 1, "111", executable);
      for (let tid = 100; tid < 230; tid++) f.task(100, tid, String(tid), tid === 101 ? 999 : 100);
      observeDescendants(100, new Set(), new Map(), f.output, f.proc);
      const packet = f.samples()[0];
      assert.equal(packet.limited, true);
      assert.ok(packet.tasks.length <= 128);
      assert.ok(packet.unavailable.includes(101));
      assert.ok(packet.tasks.every((item) => item.tgid === 100 && item.tid !== 101));
      fs.writeFileSync(path.join(actor, "maps"), "x".repeat(262_145));
      observeDescendants(100, new Set(), new Map(), f.output, f.proc);
      assert.equal(f.summary().gaps[0].identity, "UNKNOWN");
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  },
);

void test(
  "unreadable leaders remain UNKNOWN and process/packet caps retain partial coverage",
  unix,
  () => {
    const f = fixture();
    try {
      observeDescendants(100, new Set(), new Map(), f.output, f.proc);
      assert.deepEqual(f.summary().gaps, [
        { pid: 100, identity: "UNKNOWN", unavailable: "ProcessStatUnavailable" },
      ]);
      const executable = f.image("native-tool");
      for (let pid = 100; pid < 230; pid++) {
        f.actor(pid, pid === 100 ? 1 : 100, String(pid), executable);
        f.task(pid, pid, String(pid));
      }
      observeDescendants(100, new Set(), new Map(), f.output, f.proc);
      assert.equal(f.summary().limited, true);
      assert.ok(f.samples().length <= 128);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  },
);

void test(
  "long process sampling keeps late task identities after the original128-packet frontier",
  unix,
  () => {
    const f = fixture();
    try {
      const executable = f.image("native-tool");
      f.actor(100, 1, "111", executable);
      f.task(100, 100, "111");
      const seen = new Set<string>();
      const images = new Map();
      for (let index = 0; index < 300; index++) {
        if (index > 0)
          fs.rmSync(path.join(f.proc, "100/task", String(199 + index)), { recursive: true });
        f.task(100, 200 + index, String(index + 1));
        observeDescendants(100, seen, images, f.output, f.proc);
        assert.ok(f.samples().length <= 128);
      }
      const retained = f.samples();
      assert.ok(
        retained.some((packet) => packet.tasks.some((task) => task.tid === 499)),
        "the late original actor/task must remain observable after128 unique samples",
      );
      assert.equal(retained.length, 128);
      assert.ok(retained.some((packet) => packet.tasks.some((task) => task.tid === 200)));
      const summary = f.summary() as Summary & {
        retention: {
          observedPackets: number;
          droppedPackets: number;
          terminalObservation: { packetWrites: number; partial: boolean };
        };
      };
      assert.equal(summary.limited, true);
      assert.equal(summary.retention.observedPackets, 300);
      assert.equal(summary.retention.droppedPackets, 172);
      observeDescendants(100, seen, images, f.output, f.proc, "terminal");
      const terminal = f.summary() as typeof summary;
      assert.equal(terminal.retention.terminalObservation.packetWrites, 1);
      assert.equal(terminal.retention.terminalObservation.partial, true);
      assert.equal(f.samples().length, 128);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  },
);

void test(
  "real Linux procfs joins a non-Node child and its task to one physical image",
  { skip: process.platform !== "linux" },
  async () => {
    const root = fs.mkdtempSync(path.join(tmpdir(), "vize-real-process-identity-"));
    const child = spawn("/bin/sleep", ["10"], { stdio: "ignore" });
    const finished = new Promise<void>((resolve) => child.once("exit", () => resolve()));
    try {
      await new Promise<void>((resolve, reject) => {
        child.once("spawn", () => resolve());
        child.once("error", reject);
      });
      assert.ok(child.pid);
      observeDescendants(child.pid, new Set(), new Map(), root);
      const files = fs.readdirSync(path.join(root, "processes"));
      const packet = JSON.parse(
        fs.readFileSync(path.join(root, "processes", files[0]), "utf8"),
      ) as Sample;
      assert.equal(packet.pid, child.pid);
      assert.equal(packet.parentPid, process.pid);
      assert.ok(packet.parentStartTicks);
      assert.equal(packet.executable, fs.readlinkSync(`/proc/${child.pid}/exe`));
      assert.equal(
        packet.executableIdentity.sha256,
        createHash("sha256")
          .update(fs.readFileSync(`/proc/${child.pid}/exe`))
          .digest("hex"),
      );
      assert.ok(packet.tasks.some((item) => item.tid === child.pid && item.tgid === child.pid));
    } finally {
      child.kill();
      await finished;
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
