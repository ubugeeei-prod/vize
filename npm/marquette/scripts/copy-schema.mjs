import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const schemaSource = path.resolve(
  packageRoot,
  "../../crates/vize_marquette/schema/application-contract.schema.json",
);
const schemaOutput = path.join(packageRoot, "dist/application-contract.schema.json");

fs.copyFileSync(schemaSource, schemaOutput);
