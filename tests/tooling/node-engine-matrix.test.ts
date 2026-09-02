import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const supportedNodeMajors = [22, 24] as const;

interface PackageJson {
  devEngines?: {
    runtime?: {
      name?: string;
      onFail?: string;
      version?: string;
    };
  };
  engines?: Record<string, string>;
  name?: string;
  private?: boolean;
}

test("published npm packages declare Node engines covered by CI", () => {
  const failures: string[] = [];

  for (const { packageDir, packageJson } of readNpmPackages()) {
    if (packageJson.private === true || packageJson.engines?.vscode != null) {
      continue;
    }

    const engine = packageJson.engines?.node;
    if (engine == null) {
      failures.push(`${packageDir}: missing engines.node`);
      continue;
    }

    const floor = parseNodeEngineFloor(engine);
    if (floor == null) {
      failures.push(`${packageDir}: unsupported engines.node ${engine}`);
      continue;
    }

    if (!supportedNodeMajors.includes(floor as (typeof supportedNodeMajors)[number])) {
      failures.push(`${packageDir}: Node ${floor} is not in the CI compatibility matrix`);
    }
  }

  assert.deepEqual(failures, []);
});

test("current and pinned Node runtimes are represented in the support matrix", () => {
  const currentMajor = Number.parseInt(process.versions.node.split(".")[0] ?? "", 10);
  const rootPackage = JSON.parse(
    fs.readFileSync(path.join(root, "package.json"), "utf8"),
  ) as PackageJson;
  const runtime = rootPackage.devEngines?.runtime;
  assert.ok(runtime);
  assert.equal(runtime?.name, "node");
  assert.equal(runtime?.version, "24.14.0");
  assert.equal(runtime?.onFail, "download");
  const pinnedMajor = Number.parseInt(runtime.version?.split(".")[0] ?? "", 10);

  assert.ok(supportedNodeMajors.includes(currentMajor as (typeof supportedNodeMajors)[number]));
  assert.ok(supportedNodeMajors.includes(pinnedMajor as (typeof supportedNodeMajors)[number]));
});

test("Node 22 is the public package floor", () => {
  const floors = new Map<string, number>();
  for (const { packageJson } of readNpmPackages()) {
    if (packageJson.private === true || packageJson.engines?.vscode != null) {
      continue;
    }
    const name = packageJson.name;
    const engine = packageJson.engines?.node;
    assert.ok(name);
    assert.ok(engine);
    floors.set(name, parseNodeEngineFloor(engine) ?? 0);
  }

  for (const [name, floor] of floors) {
    assert.equal(floor, 22, `${name} should stay on the Node 22 floor`);
  }
});

function readNpmPackages(): Array<{ packageDir: string; packageJson: PackageJson }> {
  return fs
    .readdirSync(path.join(root, "npm"), { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .filter((entry) => fs.existsSync(path.join(root, "npm", entry.name, "package.json")))
    .map((entry) => {
      const packageDir = path.join("npm", entry.name);
      const packageJson = JSON.parse(
        fs.readFileSync(path.join(root, packageDir, "package.json"), "utf8"),
      ) as PackageJson;
      return { packageDir, packageJson };
    });
}

function parseNodeEngineFloor(engine: string): number | null {
  const floors = engine.split("||").map((part) => parseSimpleNodeEngineRange(part.trim()));

  if (floors.length === 0 || floors.some((floor) => floor == null)) {
    return null;
  }

  return Math.min(...(floors as number[]));
}

function parseSimpleNodeEngineRange(range: string): number | null {
  const gteMatch = range.match(/^>=\s*(\d+)$/);
  if (gteMatch != null) {
    return Number.parseInt(gteMatch[1], 10);
  }

  const caretMatch = range.match(/^\^(\d+)$/);
  if (caretMatch != null) {
    return Number.parseInt(caretMatch[1], 10);
  }

  return null;
}
