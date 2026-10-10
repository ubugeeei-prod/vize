import { randomUUID } from "node:crypto";
import { rename, rm, writeFile } from "node:fs/promises";

/** Publish one complete owned report; JSON/HTML retain independent file atomicity. */
export async function writeSessionReport(file: string, bytes: string): Promise<void> {
  const temporary = `${file}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, bytes, { flag: "wx" });
    await rename(temporary, file);
  } finally {
    await rm(temporary, { force: true });
  }
}
