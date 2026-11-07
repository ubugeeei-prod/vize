import fs from "node:fs";
import path from "node:path";

// Readers see the previous whole record or its whole replacement, never a
// truncated marker. Staging stays in the same owned output filesystem.
export function publishWorkerRecord(record) {
  const staging = fs.mkdtempSync(path.join(record.output, ".worker-process-"));
  const pending = path.join(staging, "record.json");
  try {
    fs.writeFileSync(pending, JSON.stringify(record, null, 2) + "\n", { flag: "wx" });
    fs.renameSync(pending, path.join(record.output, "worker-process.json"));
  } finally {
    fs.rmSync(staging, { recursive: true, force: true });
  }
}
