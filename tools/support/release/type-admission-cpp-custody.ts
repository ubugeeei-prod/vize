import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { authenticate, image, record, string } from "./public_canonical_sdk/io.ts";
import { verifyPublicProvider } from "./public-provider.ts";

const [workspaceArg, version, mode, stateFile] = process.argv.slice(2);
assert.ok(workspaceArg && version && stateFile);
assert.ok(mode === "before" || mode === "after");
for (const key of ["GH_TOKEN", "GITHUB_TOKEN", "NODE_AUTH_TOKEN", "NPM_TOKEN"])
  assert.ok(!process.env[key], key + " must not reach published consumer authentication");
const workspace = fs.realpathSync(workspaceArg),
  output = path.resolve(stateFile);
assert.ok(!output.startsWith(workspace + path.sep));
const authenticated = authenticate(workspace, version);
const source = string(record(authenticated.cpp.releaseIdentity).sourceSha);
const genuine = await verifyPublicProvider(version, source, authenticated.native);
const files: { relative: string; bytes: number; sha256: string }[] = [];
function retain(directory: string, recursive: boolean): void {
  for (const entry of fs
    .readdirSync(directory, { withFileTypes: true })
    .sort((a, b) => a.name.localeCompare(b.name))) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (recursive) retain(file, true);
      continue;
    }
    assert.ok(entry.isFile(), "original CPP input/evidence must be an ordinary file");
    const actual = image(file);
    files.push({
      relative: path.relative(workspace, file),
      bytes: actual.bytes,
      sha256: actual.sha256,
    });
  }
}
retain(workspace, false);
retain(path.join(workspace, "src"), true);
assert.equal(files.filter((item) => item.relative.startsWith("src" + path.sep)).length, 7);
const snapshot = {
  version,
  source,
  workspace,
  files,
  native: authenticated.native,
  backend: image(authenticated.provider),
  cli: image(authenticated.cli),
  nativeIdentity: record(authenticated.metadata.nativeIdentity),
  genuine,
};
if (mode === "before") {
  fs.writeFileSync(
    output,
    JSON.stringify({ schema: "vize.typeAdmission.cppCustody.v1", snapshot }, null, 2) + "\n",
    { flag: "wx" },
  );
} else {
  const previous: unknown = JSON.parse(fs.readFileSync(output, "utf8"));
  const baseline = record(previous);
  assert.equal(baseline.schema, "vize.typeAdmission.cppCustody.v1");
  assert.deepEqual(
    snapshot,
    baseline.snapshot,
    "original CPP sources, full evidence, lock, native and backend must remain unchanged",
  );
  fs.writeFileSync(
    output + ".after.json",
    JSON.stringify({ schema: baseline.schema, snapshot, unchanged: true }, null, 2) + "\n",
    { flag: "wx" },
  );
}
