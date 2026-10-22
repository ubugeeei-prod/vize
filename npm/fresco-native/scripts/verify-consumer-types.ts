import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8"));
const publishedTypeFiles = [manifest.main, manifest.types];

for (const file of publishedTypeFiles) {
  if (typeof file !== "string" || !manifest.files.includes(file)) {
    throw new Error(`Fresco native consumer type entry is not published: ${String(file)}`);
  }
}

const consumerDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fresco-native-consumer-"));
try {
  const stagedPackageDir = path.join(consumerDir, "node_modules", "@vizejs", "fresco-native");
  const stagedFixtureDir = path.join(consumerDir, "tests", "types");
  fs.mkdirSync(stagedPackageDir, { recursive: true });
  fs.mkdirSync(stagedFixtureDir, { recursive: true });
  fs.copyFileSync(
    path.join(packageDir, "package.json"),
    path.join(stagedPackageDir, "package.json"),
  );
  for (const file of publishedTypeFiles) {
    fs.copyFileSync(path.join(packageDir, file), path.join(stagedPackageDir, file));
  }
  fs.copyFileSync(
    path.join(packageDir, "tests", "types", "consumer.ts"),
    path.join(stagedFixtureDir, "consumer.ts"),
  );
  fs.copyFileSync(
    path.join(packageDir, "tsconfig.types.json"),
    path.join(consumerDir, "tsconfig.types.json"),
  );

  const command = process.platform === "win32" ? "vp.cmd" : "vp";
  const result = spawnSync(
    command,
    ["exec", "tsc", "--noEmit", "-p", path.join(consumerDir, "tsconfig.types.json")],
    { cwd: packageDir, encoding: "utf8", shell: process.platform === "win32" },
  );
  if (result.error != null) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `Fresco native consumer type check failed\n${result.stderr}\n${result.stdout}`.trim(),
    );
  }
  console.log("Fresco native published package types compile for an isolated consumer.");
} finally {
  fs.rmSync(consumerDir, { recursive: true, force: true });
}
