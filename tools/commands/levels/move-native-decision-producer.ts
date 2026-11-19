import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? process.cwd());
const source = path.join(root, "davinci/vize_l2_to_l3/src/decision.rs");
const target = path.join(root, "davinci/vize_l3/src/decision/build.rs");
const facade =
  "//! Compatibility exports of the sole native decision producer owned by L3.\n\n" +
  "pub use vize_l3::decision::{DecisionBuildError, NativeAnalysis, build_decisions};\n";
if (fs.existsSync(target)) {
  if (fs.existsSync(source) && fs.readFileSync(source, "utf8") !== facade) {
    throw new Error("Both native producer paths exist");
  }
} else if (fs.existsSync(source)) {
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.renameSync(source, target);
} else {
  throw new Error("Neither native producer path exists");
}
