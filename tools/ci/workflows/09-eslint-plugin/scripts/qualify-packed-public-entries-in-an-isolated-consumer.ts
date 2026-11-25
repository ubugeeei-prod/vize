import { requiredEnv } from "../../../lib/scripts/files.ts";
import { writeFileSync, mkdirSync, symlinkSync, readdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { resolve, join } from "node:path";
const root = process.cwd();
const consumer = requiredEnv("VIZE_ESLINT_PACKED_CONSUMER");
writeFileSync(join(consumer, "package.json"), JSON.stringify({ private: true, type: "module" }));
const packages = readdirSync(requiredEnv("VIZE_ESLINT_EVIDENCE"))
  .filter((file) => file.endsWith(".tgz"))
  .map((file) => resolve(requiredEnv("VIZE_ESLINT_EVIDENCE"), file));
execFileSync(
  "vp",
  [
    "exec",
    "pnpm",
    "add",
    "--ignore-scripts",
    "--no-optional",
    "--save-exact",
    ...packages,
    "eslint@10.13.0",
    "typescript@6.0.3",
  ],
  { cwd: consumer, stdio: "inherit" },
);
mkdirSync(join(consumer, "node_modules/@vizejs"), { recursive: true });
symlinkSync(join(root, "npm/native"), join(consumer, "node_modules/@vizejs/native"), "dir");
