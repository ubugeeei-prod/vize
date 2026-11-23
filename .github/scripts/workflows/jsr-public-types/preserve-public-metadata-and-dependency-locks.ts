import { requiredEnv } from "../shared/files.ts";
import fs from "node:fs";
import { createHash } from "node:crypto";
const metadata = JSON.parse(fs.readFileSync("public-metadata.json", "utf8"));
if (!metadata.versions[requiredEnv("PUBLIC_JSR_VERSION")]?.createdAt)
  throw new Error("Version missing from public JSR");
const files = fs.readdirSync(".").filter((name) => /(?:\.log|\.json|\.lock|\.txt)$/.test(name));
const rows = files.map((name) => ({
  name,
  sha256: createHash("sha256").update(fs.readFileSync(name)).digest("hex"),
}));
fs.writeFileSync(
  "evidence-sha256.json",
  JSON.stringify(
    { version: process.env.PUBLIC_JSR_VERSION, observedAt: new Date().toISOString(), files: rows },
    null,
    2,
  ),
);
