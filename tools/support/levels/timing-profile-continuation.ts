import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { hostImport, originalLawSHA, timingMoves } from "./move-timing-observer-host.ts";

const curator = "crates/vize_curator/src/inspector/stages/profile.rs";
const oldCuratorImport = "use vize_l0::pass::TimingObserver;";
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
type Reader = (file: string) => string;

/** Current physical law path; the original replay contract remains unchanged. */
export const currentTimingOraclePath = (file: string) =>
  file === timingMoves[1][0] ? timingMoves[1][1] : file;

/** Checks-only projection of the exact import move into the frozen export law. */
export function withTimingProfileContinuation(read: Reader): Reader {
  const source = read(curator);
  if (!source.includes(hostImport)) return read;
  assert.equal(
    digest(source),
    "1e3b5cc15cf6aa4610320b36e6ef361ba8d7adf423e79f88fee6a9f99493894f",
    "unexpected timing host caller body",
  );
  assert.equal(source.split(hostImport).length, 2, "unexpected timing host import");
  const projected = source.replace(
    hostImport + "\nuse vize_l0::FxHashMap;",
    "use vize_l0::FxHashMap;\n" + oldCuratorImport,
  );
  assert.equal(
    digest(projected),
    "7e1ad6a552d23ff11d409b32c135047857bbff88b35bfe058345af34aa5cd533",
    "unexpected original Curator body",
  );
  let absent = false;
  try {
    read(timingMoves[1][0]);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    absent = true;
  }
  assert.ok(absent, "unexpected duplicate original timing law");
  const law = read(timingMoves[1][1]);
  assert.equal(
    digest(law),
    "bc0e294183226f946b1240571db41073cf6b3f2416fd7d68b67df16192f46296",
    "unexpected whole moved timing law",
  );
  const original = law.replace(hostImport, "use vize_l0::pass::observer::TimingObserver;");
  assert.equal(digest(original), originalLawSHA, "unexpected original timing law bytes");
  return (file) =>
    file === curator ? projected : file === timingMoves[1][0] ? original : read(file);
}
