import { existsSync, mkdirSync, readFileSync, renameSync } from "node:fs";
import path from "node:path";

const source = path.resolve("davinci/vize_l1/src/embed/syntax/shapes.rs");
const target = path.resolve("vendor/oxc_parser/src/embedding/shapes.rs");
if (existsSync(target)) {
  // The functional adapter restores only authored views at the original path.
  if (
    existsSync(source) &&
    /fn (?:context_hole|contains_module_declaration|valid_wrapper)/u.test(
      readFileSync(source, "utf8"),
    )
  ) {
    throw new Error("Both language-policy copies exist; inspect the source before relocating");
  }
} else {
  if (!existsSync(source)) throw new Error("Original embedding shape source is absent");
  mkdirSync(path.dirname(target), { recursive: true });
  renameSync(source, target);
}
