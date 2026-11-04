import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { typecheckHarness } from "./template-alias-paired-11822-declarations.ts";
import { requiredDependencyDirectory } from "./template-alias-paired-11822-packages.ts";

const directory = dirname(fileURLToPath(import.meta.url));
const source = resolve(directory, "../../..");
const api = join(
  requiredDependencyDirectory(join(source, "npm/cli"), "typescript"),
  "lib/typescript.js",
);
const nodeTypes = requiredDependencyDirectory(source, "@types/node");

typecheckHarness(directory, api, nodeTypes);
const result = spawnSync(
  process.execPath,
  [
    "--test",
    join(directory, "template-alias-paired-11822.test.ts"),
    join(directory, "template-alias-paired-11822-declarations.test.ts"),
  ],
  {
    cwd: source,
    env: { ...process.env, TEMPLATE_ALIAS_TYPESCRIPT_API: api },
    stdio: "inherit",
  },
);
assert.equal(result.error, undefined, "functional contract process failed to start");
assert.equal(result.signal, null, "functional contract process was interrupted");
assert.equal(result.status, 0, "functional campaign qualification failed");
