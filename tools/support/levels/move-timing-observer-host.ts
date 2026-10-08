import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const timingMoves = [
  ["davinci/vize_l0/src/pass/observer/timing.rs", "crates/vize_carton/src/timing_observer.rs"],
  [
    "davinci/vize_l0/tests/pass_observer_timing.rs",
    "crates/vize_carton/tests/pass_observer_timing.rs",
  ],
] as const;
export const originalObserverSHA =
  "cca4d5f3a77ea4a849823492b3f1123733876e3993d3aeb4ad731fea33c59f15";
export const originalLawSHA = "887262892647fd725260d1fa2b205aa914608049f269395399dbb5322d2fa8f7";
export const hostImport = "use vize_carton::timing_observer::TimingObserver;";
export const hostCallers = [
  "crates/vize/src/commands/davinci_ice.rs",
  "crates/vize_curator/src/inspector/stages/profile.rs",
] as const;
const oldObserverImports =
  "mod walk;\n\npub use walk::WalkTiming;\n\nuse vize_l0::profiler::global_profiler;\n\nuse super::{FailEvent, PassEvent, PassObserver, Pipeline};";
const newObserverImports =
  "use vize_l0::pass::{FailEvent, PassEvent, PassObserver, Pipeline, WalkTiming};\nuse vize_l0::profiler::global_profiler;";
export const portableWrapper =
  "//! Portable fused-walk state. The host clock observer lives in Carton.\n\nmod walk;\n\npub use walk::WalkTiming;\n";
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
const once = (text: string, needle: string) => assert.equal(text.split(needle).length, 2, needle);

/** Preflight the complete moved bodies and every known import before any write. */
export function timingIntegrationPlan(read: (file: string) => string) {
  const plan = new Map<string, string>();
  const facade = read("crates/vize_carton/src/lib.rs");
  const pending = !facade.includes("pub mod timing_observer;\n");
  const observer = read(timingMoves[0][1]);
  const law = read(timingMoves[1][1]);
  if (pending) {
    assert.equal(digest(observer), originalObserverSHA);
    assert.equal(digest(law), originalLawSHA);
    once(observer, oldObserverImports);
    plan.set(
      timingMoves[0][1],
      observer
        .replace(oldObserverImports, newObserverImports)
        .replaceAll("super::PassEvent", "vize_l0::pass::PassEvent"),
    );
    plan.set(timingMoves[0][0], portableWrapper);
    plan.set("crates/vize_carton/src/lib.rs", facade + "\npub mod timing_observer;\n");
  } else {
    assert.equal(
      digest(observer),
      "85552a54ac8d9a0008970456f9bf9373d34941ffe5102b191b430c13e947a195",
      "unexpected complete host timing observer",
    );
    once(facade, "pub mod timing_observer;\n");
    assert.equal(read(timingMoves[0][0]), portableWrapper);
    once(observer, newObserverImports);
    assert.equal(
      digest(
        observer
          .replace(newObserverImports, oldObserverImports)
          .replaceAll("vize_l0::pass::PassEvent", "super::PassEvent"),
      ),
      originalObserverSHA,
    );
    assert.equal(
      digest(law.replace(hostImport, "use vize_l0::pass::observer::TimingObserver;")),
      originalLawSHA,
    );
  }
  const observerExports = "davinci/vize_l0/src/pass/observer.rs";
  const exports = read(observerExports);
  if (pending) {
    once(exports, "pub use timing::{TimingObserver, WalkTiming};");
    plan.set(
      observerExports,
      exports.replace(
        "pub use timing::{TimingObserver, WalkTiming};",
        "pub use timing::WalkTiming;",
      ),
    );
  } else
    assert.ok(
      exports.includes("pub use timing::WalkTiming;") &&
        !exports.includes("TimingObserver, WalkTiming"),
    );
  for (const file of [
    "davinci/vize_l0/src/pass.rs",
    hostCallers[0],
    "davinci/vize_l0/tests/dump_collector.rs",
    "davinci/vize_l0/tests/remark_zero_cost.rs",
  ]) {
    const text = read(file);
    if (pending) {
      once(text, "TimingObserver,");
      assert.ok(!text.includes(hostImport));
      const after = text.replace(/TimingObserver,[ \t]*/u, "");
      assert.ok(file === "davinci/vize_l0/src/pass.rs" || /^use /mu.test(after));
      plan.set(
        file,
        file === "davinci/vize_l0/src/pass.rs"
          ? after
          : after.replace(/^use /mu, hostImport + "\nuse "),
      );
    } else {
      assert.ok(!text.includes("TimingObserver,"));
      if (file !== "davinci/vize_l0/src/pass.rs") once(text, hostImport);
    }
  }
  for (const [file, before] of [
    [hostCallers[1], "use vize_l0::pass::TimingObserver;"],
    [timingMoves[1][1], "use vize_l0::pass::observer::TimingObserver;"],
  ] as const) {
    const text = read(file);
    if (pending) {
      once(text, before);
      assert.ok(!text.includes(hostImport));
      plan.set(file, text.replace(before, hostImport));
    } else once(text, hostImport);
  }
  return plan;
}

export function replayTiming(phase: "moves" | "integrate" | "check", root: string) {
  if (phase === "moves") {
    for (const [before, after] of timingMoves) {
      assert.equal(fs.existsSync(path.join(root, before)), !fs.existsSync(path.join(root, after)));
      if (fs.existsSync(path.join(root, before)))
        execFileSync("git", ["mv", before, after], { cwd: root });
    }
    return;
  }
  assert.ok(!fs.existsSync(path.join(root, timingMoves[1][0])));
  const planned = timingIntegrationPlan((file) => fs.readFileSync(path.join(root, file), "utf8"));
  if (phase === "check") assert.equal(planned.size, 0);
  else for (const [file, text] of planned) fs.writeFileSync(path.join(root, file), text);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2];
  assert.ok(mode === "moves" || mode === "integrate" || mode === "check");
  assert.equal(process.argv.length, 3);
  replayTiming(mode, path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../.."));
}
