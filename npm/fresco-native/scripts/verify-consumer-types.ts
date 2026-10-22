import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface PackageManifest {
  readonly main: string;
  readonly types: string;
  readonly files: readonly string[];
}

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

const readString = (value: unknown, description: string): string => {
  if (typeof value !== "string") throw new TypeError(`${description} must be a string`);
  return value;
};

const readStringArray = (value: unknown, description: string): readonly string[] => {
  if (!Array.isArray(value)) throw new TypeError(`${description} must be an array`);
  return value.map((entry: unknown, index) => readString(entry, `${description}[${index}]`));
};

const readPackageManifest = (value: unknown): PackageManifest => {
  if (!isRecord(value)) throw new TypeError("Fresco native package manifest must be an object");
  return {
    main: readString(value.main, "Fresco native package manifest main"),
    types: readString(value.types, "Fresco native package manifest types"),
    files: readStringArray(value.files, "Fresco native package manifest files"),
  };
};

const packageDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestSource: unknown = JSON.parse(
  fs.readFileSync(path.join(packageDir, "package.json"), "utf8"),
);
const manifest = readPackageManifest(manifestSource);
const publishedTypeFiles = [manifest.main, manifest.types];

for (const file of publishedTypeFiles) {
  if (!manifest.files.includes(file)) {
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
