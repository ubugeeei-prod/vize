import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

/** perf --clockid mono and Node's Linux hrtime use CLOCK_MONOTONIC. */
export function measuredStacks(script, phases) {
  const intervals = Object.values(phases).map(({ startNs, endNs }) => [
    BigInt(startNs),
    BigInt(endNs),
  ]);
  return script.split(/\n\s*\n/u).filter((block) => {
    const timestamp = block.split("\n")[0].match(/\s(\d+)\.(\d+):\s/u);
    if (!timestamp) return false;
    const nanos = BigInt(timestamp[1]) * 1_000_000_000n + BigInt(timestamp[2].padEnd(9, "0"));
    return intervals.some(([start, end]) => start <= nanos && nanos <= end);
  });
}

/** Additional sampled invocations never enter the accepted timing medians. */
export function profileNative(run, worker, image, fixture, out, shape) {
  const perf = process.env.IMPORTED_PROPS_PERF;
  if (!perf || !["inline-1", "barrel-800"].includes(shape)) return;
  for (const threads of [1, 16]) {
    const chunk = shape === "inline-1" ? 32 : 128;
    const name = `profile-${shape}-t${threads}-c${chunk}`;
    const receipt = path.join(out, `${name}.json`);
    const data = path.join(out, `${name}.perf.data`);
    run(
      perf,
      [
        "record",
        "-e",
        "cpu-clock",
        "-F",
        "999",
        "--clockid",
        "mono",
        "--call-graph",
        "dwarf,65528",
        "-o",
        data,
        "--",
        process.execPath,
        worker,
        "native",
        image,
        fixture,
        String(chunk),
        receipt,
      ],
      out,
      name,
      { RAYON_NUM_THREADS: String(threads) },
    );
    const { cold, warm } = JSON.parse(fs.readFileSync(receipt, "utf8"));
    const script = run(perf, ["script", "--ns", "-i", data], out, `${name}-script`);
    const blocks = measuredStacks(script, { cold, warm });
    assert(blocks.length > 20, "sampled stacks must cover the measured native intervals");
    fs.writeFileSync(path.join(out, `${name}.native-stacks.txt.gz`), gzipSync(blocks.join("\n\n")));
    fs.writeFileSync(path.join(out, `${name}.all-stacks.txt.gz`), gzipSync(script));
    const self = new Map();
    const inclusive = new Map();
    for (const block of blocks) {
      const frames = block
        .split("\n")
        .slice(1)
        .map((line) =>
          line
            .trim()
            .replace(/^[a-f0-9]+\s+/u, "")
            .replace(/\+0x[0-9a-f]+(?=\s|$)/u, ""),
        )
        .filter(Boolean);
      if (frames[0]) self.set(frames[0], (self.get(frames[0]) ?? 0) + 1);
      for (const frame of new Set(frames)) inclusive.set(frame, (inclusive.get(frame) ?? 0) + 1);
    }
    const top = (counts) => [...counts].sort((a, b) => b[1] - a[1]).slice(0, 80);
    fs.writeFileSync(
      path.join(out, `${name}.summary.json`),
      JSON.stringify(
        {
          event: "cpu-clock",
          frequencyHz: 999,
          clock: "CLOCK_MONOTONIC",
          samples: blocks.length,
          cold: {
            startNs: cold.startNs,
            endNs: cold.endNs,
            wallMs: cold.wallMs,
            cpuMs: cold.cpuMs,
          },
          warm: {
            startNs: warm.startNs,
            endNs: warm.endNs,
            wallMs: warm.wallMs,
            cpuMs: warm.cpuMs,
          },
          self: top(self),
          inclusive: top(inclusive),
        },
        null,
        2,
      ),
    );
    const scriptLog = path.join(out, `${name}-script.log`);
    fs.writeFileSync(`${scriptLog}.gz`, gzipSync(fs.readFileSync(scriptLog)));
    fs.rmSync(scriptLog);
    fs.rmSync(data);
  }
}
