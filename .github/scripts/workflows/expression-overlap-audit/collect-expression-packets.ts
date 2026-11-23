import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, writeJson } from "../shared/files.ts";

import { execFileSync } from "node:child_process";
import { closeSync, openSync, writeFileSync } from "node:fs";
interface Case {
  name: string;
  require_admission: boolean;
  historical_id?: string;
}
const root = "evidence",
  guard = requiredEnv("GUARD_BINARY"),
  profiler = requiredEnv("PROFILE_BINARY");
for (const entry of readJson<Case[]>(join(root, "cases/manifest.json"))) {
  const prefix = join(root, entry.name),
    file = join(root, "cases", entry.name);
  const inspection = execFileSync(guard, ["inspect", file]);
  writeFileSync(prefix + ".admission.json", inspection);
  if (entry.require_admission)
    assert.ok((JSON.parse(inspection.toString()) as { safe: boolean }).safe, entry.name);
  const fd = openSync(prefix + ".packets.txt", "w");
  try {
    execFileSync(profiler, ["packets", file], { stdio: ["ignore", fd, "inherit"], timeout: 90000 });
  } finally {
    closeSync(fd);
  }
  if (entry.historical_id) {
    const observers = [
      ["controls", guard, ["controls", file]],
      ["program-observer", requiredEnv("HISTORICAL_OBSERVER"), [file]],
    ] as const;
    for (const [kind, binary, args] of observers) {
      const actual = execFileSync(binary, [...args], {
        timeout: 90000,
        maxBuffer: 256 * 1024 * 1024,
      });
      assert.deepEqual(
        JSON.parse(actual.toString()),
        readJson(
          join(root, "cases/historical-packets", entry.historical_id + "." + kind + ".json"),
        ),
        entry.historical_id + " " + kind,
      );
      writeFileSync(prefix + ".historical-" + kind + ".json", actual);
    }
  }
  const timings = Array.from(
    { length: 5 },
    () =>
      JSON.parse(
        execFileSync(profiler, ["time", file, "1"], { encoding: "utf8", timeout: 30000 }),
      ) as unknown,
  );
  writeJson(prefix + ".timings.json", timings);
}
