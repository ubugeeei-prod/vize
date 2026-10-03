import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export function levelInstructionSuites(root) {
  return [
    ["davinci_harness", "selfcheck"],
    ["vize_armature", "davinci"],
    ["vize_croquis", "davinci"],
    ["vize_atelier_core", "davinci"],
    ["vize_atelier_dom", "davinci"],
    ["vize_atelier_vapor", "davinci"],
    ["vize_atelier_ssr", "davinci"],
    ["vize_l0", "pass_runtime"],
    ["vize_l0", "fact_runtime"],
    ["vize_l1_to_l2", "l1_to_l2_storage"],
    ["vize_patina", "davinci_markup"],
    ["vize_musea", "davinci_art"],
  ].map(([pkg, bench]) => {
    // Target rename is bijective: probe ids, exact input bytes, windows and caps
    // stay fixed. Support either side until the move-only rename merges.
    if (pkg !== "vize_l1_to_l2") return [pkg, bench];
    const matches = [bench, "davinci_storage"].filter((name) =>
      ["crates", "davinci"].some((directory) =>
        fs.existsSync(path.join(root, directory, pkg, "benches", `${name}.rs`)),
      ),
    );
    assert.equal(matches.length, 1, "level storage target rename must be bijective");
    return [pkg, matches[0]];
  });
}
