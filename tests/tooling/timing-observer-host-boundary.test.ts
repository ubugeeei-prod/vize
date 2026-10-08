import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  hostCallers,
  hostImport,
  originalLawSHA,
  timingIntegrationPlan,
  timingMoves,
} from "../../tools/support/levels/move-timing-observer-host.ts";
import { withTimingProfileContinuation } from "../../tools/support/levels/timing-profile-continuation.ts";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";

const read = (file: string) => fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8");
const digest = (text: string) => createHash("sha256").update(text).digest("hex");

test("whole original observer and law replay while portable walk stays in L0", () => {
  assert.equal(timingIntegrationPlan(read).size, 0);
  assert.equal(
    digest(
      read(timingMoves[1][1]).replace(hostImport, "use vize_l0::pass::observer::TimingObserver;"),
    ),
    originalLawSHA,
  );
  assert.equal(
    digest(read("davinci/vize_l0/src/pass/observer/timing/walk.rs")),
    "3887b2faed4ef43a4a8e34fe64f2fe74eda1f12642f65561fd8979be3d098a4e",
  );
  assert.equal(
    digest(read("davinci/vize_l0/tests/walk_timing.rs")),
    "1ea1009073ec3573f1c9e43ef16d77fe60e85b762bc3f298ea2806cb2161540f",
  );
  for (const file of [timingMoves[0][1], timingMoves[1][1]])
    assert.throws(() =>
      timingIntegrationPlan(
        (path) => read(path) + (path === file ? "\nfn unexpected_body() {}\n" : ""),
      ),
    );
});

test("partial transformed observer documentation cannot pass replay", () => {
  assert.throws(
    () =>
      timingIntegrationPlan((file) => {
        const text = read(file);
        return file === timingMoves[0][1]
          ? text.replace(
              "vize_l0::pass::PassEvent::is_group_entry",
              "super::PassEvent::is_group_entry",
            )
          : text;
      }),
    /complete host timing observer/u,
  );
});

test("only the two existing normal host imports are admitted", () => {
  for (const file of hostCallers) {
    assert.doesNotMatch(withoutHostRuntimeReferences(read(file), file), /\bvize_carton\b/u);
    for (const extra of [
      hostImport,
      "use vize_carton::String;",
      "use vize_carton::timing_observer::*;",
      "use vize_carton::timing_observer::UnknownObserver;",
    ])
      assert.match(
        withoutHostRuntimeReferences(read(file) + "\n" + extra, file),
        /\bvize_carton\b/u,
      );
  }
  for (const file of ["davinci/vize_l0/src/lib.rs", "crates/vize_curator/src/other.rs"])
    assert.match(withoutHostRuntimeReferences(hostImport, file), /\bvize_carton\b/u);
});

test("frozen profile replay projects only the complete timing import move", () => {
  const projected = withTimingProfileContinuation(read);
  assert.equal(digest(projected(timingMoves[1][0])), originalLawSHA);
  const curator = hostCallers[1];
  assert.equal(
    digest(projected(curator)),
    "7e1ad6a552d23ff11d409b32c135047857bbff88b35bfe058345af34aa5cd533",
  );
  assert.throws(
    () =>
      withTimingProfileContinuation((file) =>
        file === timingMoves[1][0] ? projected(file) : read(file),
      ),
    /duplicate/u,
  );
  for (const file of [curator, timingMoves[1][1]])
    assert.throws(
      () =>
        withTimingProfileContinuation(
          (path) => read(path) + (path === file ? "\nfn unexpected_body() {}\n" : ""),
        ),
      /unexpected/u,
    );
  assert.throws(
    () =>
      withTimingProfileContinuation((file) => {
        if (file === timingMoves[1][0])
          throw Object.assign(new Error("permission denied"), { code: "EACCES" });
        return read(file);
      }),
    /permission denied/u,
  );
});
