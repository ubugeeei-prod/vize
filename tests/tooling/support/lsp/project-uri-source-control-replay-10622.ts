import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { object, sha256 } from "../project-uri-membership-10622.ts";
import { verifyRecordedTrial } from "./project-uri-source-control-recorded-10622.ts";

const [artifactArgument, archive, output] = process.argv.slice(2);
assert.equal(process.argv.length, 5);
assert.ok(artifactArgument && archive && output);
const root = fs.realpathSync(
  path.join(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
);
const artifact = fs.realpathSync(artifactArgument);
const hash = createHash("sha256");
for await (const bytes of fs.createReadStream(archive)) hash.update(bytes);
const digest = hash.digest("hex");
const zip = digest === "8ab9fc5f162cb42f7d0f5093c275f34791fc9765847e0f26598f4b38cd8ac2f6";
assert.ok(
  zip || digest === "c814b4e9b87514ddb5a10c3f60ab0868de507ff68b9369ed335597f322c3dfbf",
  "the exact original GitHub artifact or verified preserved archive is required",
);
const archived = new Map<string, Buffer>();
const read = (file: string) => {
  const prior = archived.get(file);
  if (prior) {
    assert.deepEqual(fs.readFileSync(file), prior);
    return prior;
  }
  const relative = path.relative(artifact, file);
  assert.ok(!relative.startsWith("..") && !path.isAbsolute(relative));
  assert.equal(fs.lstatSync(file).isSymbolicLink(), false);
  const member = zip ? relative : `vize-10672-df4b-artifact/${relative}`;
  const original = execFileSync(
    zip ? "unzip" : "tar",
    zip ? ["-p", archive, member] : ["-xOf", archive, member],
    { maxBuffer: 32 * 1024 * 1024 },
  );
  const bytes = fs.readFileSync(file);
  assert.deepEqual(bytes, original, `exact archived bytes ${relative}`);
  archived.set(file, bytes);
  return bytes;
};
const metadataRoot = path.join(
  artifact,
  "vize/vize/target/vize-tests/project-uri-source-control-10622",
);
const statuses = JSON.parse(read(path.join(metadataRoot, "trial-statuses.json")).toString());
assert.deepEqual(
  statuses.map(
    (row: { role: unknown; trial: unknown; status: unknown; signal: unknown; error: unknown }) => [
      row.role,
      row.trial,
      row.status,
      row.signal,
      row.error,
    ],
  ),
  [
    ["control", "1", 7, null, null],
    ["candidate", "1", 7, null, null],
    ["candidate", "2", 7, null, null],
    ["control", "2", 7, null, null],
  ],
);
const trials = [];
for (const role of ["control", "candidate"]) {
  const producer = JSON.parse(
    read(path.join(metadataRoot, "producers", `${role}.json`)).toString(),
  );
  const sessions = path.join(
    artifact,
    "_temp/uri-source-control-producers",
    role,
    "target/differential/lsp-sessions",
  );
  const directories = fs.readdirSync(sessions, { withFileTypes: true });
  assert.equal(directories.length, 2);
  assert.ok(directories.every((entry) => entry.isDirectory() && !entry.isSymbolicLink()));
  for (const trial of ["1", "2"]) {
    const identities = JSON.parse(
      read(path.join(metadataRoot, role, `trial-${trial}`, "identities.json")).toString(),
    );
    assert.equal(identities.role, role);
    assert.equal(identities.trial, trial);
    const matching = directories.filter((entry) => {
      const client = decodeFrames(read(path.join(sessions, entry.name, "client.bin"))).messages;
      const init = client.find((packet) => packet.method === "initialize");
      assert.ok(init);
      const params = object(init.params);
      assert.equal(typeof params.rootUri, "string");
      assert.ok(typeof params.rootUri === "string");
      return fileURLToPath(params.rootUri) === identities.workspace;
    });
    assert.equal(matching.length, 1);
    const directory = path.join(sessions, matching[0].name);
    for (const file of ["observation.json", "client.bin", "server.bin", "stderr.bin"])
      read(path.join(directory, file));
    trials.push({ role, trial, ...verifyRecordedTrial(root, producer, identities, directory) });
  }
}
assert.equal(trials.length, 4);
for (const field of ["compiler", "cargo", "locks", "runner"]) {
  const first = object(trials[0].producer)[field];
  for (const trial of trials)
    assert.deepEqual(
      object(trial.producer)[field],
      first,
      `same recorded ${field} for all four trials`,
    );
}
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(
  output,
  JSON.stringify(
    {
      authority: "archived-dual-root-record",
      auditor: {
        revision: execFileSync("git", ["rev-parse", "HEAD"], {
          cwd: root,
          encoding: "utf8",
        }).trim(),
        file: import.meta.filename,
        sha256: sha256(fs.readFileSync(import.meta.filename)),
      },
      archive: { path: archive, sha256: digest },
      originalFailingDriverStatuses: statuses,
      trials,
      genericSourceWitnessAdmission: false,
      producerBinaryRehashed: false,
      semanticAcceptance: false,
      qualification:
        "Exact recorded producer receipts, separate historical observer identity and all closed outer bytes only. Original generic empty-witness failure, one-owner law and native teardown failures remain unqualified and unchanged.",
    },
    null,
    2,
  ) + "\n",
);
console.log(
  JSON.stringify({
    output,
    trials: trials.length,
    archiveSha256: digest,
    semanticAcceptance: false,
  }),
);
