import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const schemas = ["application-contract.schema.json", "test-run-evidence.schema.json"];

for (const schema of schemas) {
  fs.copyFileSync(
    path.resolve(packageRoot, "../../crates/vize_marquette/schema", schema),
    path.join(packageRoot, "dist", schema),
  );
}
