import assert from "node:assert/strict";
import fs from "node:fs";

type Identity = { pid: number; birth_ticks: number };

function stat(file: string) {
  const raw = fs.readFileSync(file, "utf8");
  const fields = raw
    .slice(raw.lastIndexOf(") ") + 2)
    .trim()
    .split(/\s+/u);
  for (const offset of [11, 12, 19]) assert.match(fields[offset], /^\d+$/u);
  return {
    raw,
    state: fields[0],
    userTicks: fields[11],
    systemTicks: fields[12],
    birth_ticks: Number(fields[19]),
  };
}

/** Raw kernel observations bracket requests without entering their timed window. */
export function observeWork(
  sample: { processes: Identity[] },
  parent: { pid: number; executable: string },
) {
  assert.ok(sample.processes.some((entry) => entry.pid === parent.pid));
  return sample.processes.map((entry) => {
    try {
      const current = stat(`/proc/${entry.pid}/stat`);
      assert.equal(current.birth_ticks, entry.birth_ticks, "sampled process life changed");
      const executable = fs.realpathSync(`/proc/${entry.pid}/exe`);
      if (entry.pid === parent.pid) assert.equal(executable, parent.executable);
      const io = fs.readFileSync(`/proc/${entry.pid}/io`, "utf8");
      const tasks = fs
        .readdirSync(`/proc/${entry.pid}/task`)
        .toSorted()
        .map((tid) => {
          assert.match(tid, /^\d+$/u);
          try {
            return {
              tid: Number(tid),
              name: fs.readFileSync(`/proc/${entry.pid}/task/${tid}/comm`, "utf8"),
              ...stat(`/proc/${entry.pid}/task/${tid}/stat`),
            };
          } catch (failure) {
            // A thread can leave between enumeration and observation. Retain the
            // missed observation rather than fabricate zero work for that life.
            return { tid: Number(tid), error: String(failure) };
          }
        });
      const finished = stat(`/proc/${entry.pid}/stat`);
      assert.equal(finished.birth_ticks, entry.birth_ticks, "observed process life changed");
      assert.equal(fs.realpathSync(`/proc/${entry.pid}/exe`), executable);
      return { ...entry, executable, current, io, tasks, finished };
    } catch (failure) {
      // The parent owns the measured request. Its identity must remain exact;
      // native-child departures are recorded as lifecycle observations.
      if (entry.pid === parent.pid) throw failure;
      return { ...entry, error: String(failure) };
    }
  });
}
