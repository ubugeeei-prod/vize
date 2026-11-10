import { readFileSync, readdirSync } from "node:fs";

const kib = (status: string, field: string): number | undefined => {
  const value = new RegExp(`^${field}:\\s+(\\d+) kB`, "mu").exec(status)?.[1];
  return value === undefined ? undefined : Number(value);
};

/** Poll /proc for the high-water marks of vize and of every checker it spawned. */
export function sampleTree(rootPid: number): {
  stop: () => { vize?: number; children?: number };
} {
  const peaks = new Map<number, { name: string; hwm: number }>();
  // Each thread lists the children it spawned, and vize spawns from workers.
  const children = (pid: number): number[] => {
    try {
      return readdirSync(`/proc/${pid}/task`).flatMap((task) => {
        try {
          return readFileSync(`/proc/${pid}/task/${task}/children`, "utf8")
            .trim()
            .split(/\s+/u)
            .filter(Boolean)
            .map(Number);
        } catch {
          return [];
        }
      });
    } catch {
      return [];
    }
  };
  const tick = () => {
    const stack = [rootPid];
    while (stack.length) {
      const pid = stack.pop() ?? 0;
      try {
        const status = readFileSync(`/proc/${pid}/status`, "utf8");
        const name = /^Name:\s+(\S+)/mu.exec(status)?.[1] ?? "";
        const hwm = kib(status, "VmHWM") ?? 0;
        if (hwm >= (peaks.get(pid)?.hwm ?? 0)) peaks.set(pid, { name, hwm });
      } catch {
        // exited between listing and reading
      }
      stack.push(...children(pid));
    }
  };
  const timer = setInterval(tick, 20);
  return {
    stop: () => {
      clearInterval(timer);
      let vize: number | undefined;
      let sum = 0;
      for (const { name, hwm } of peaks.values()) {
        if (name === "time") continue;
        if (name.startsWith("vize")) vize = Math.max(vize ?? 0, hwm);
        else sum += hwm;
      }
      return { vize: vize === undefined ? undefined : vize / 1024, children: sum / 1024 };
    },
  };
}
