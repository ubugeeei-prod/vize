import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";

const capture = `${process.env.RUNNER_TEMP}/component-input-ownership/generic-regression`;
const messages = readFileSync(`${capture}/build.jsonl`, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line));
assert.equal(messages.at(-1).reason, "build-finished");
assert.equal(messages.at(-1).success, true);

const targets = messages.filter(
  (message) =>
    message.reason === "compiler-artifact" &&
    message.target.name === "integration" &&
    message.target.kind.join() === "test" &&
    message.executable,
);
assert.equal(targets.length, 1);

const binary = realpathSync(targets[0].executable);
const hash = (file) => createHash("sha256").update(readFileSync(file)).digest("hex");
const binarySha256 = hash(binary);
const run = (name, args) => {
  const result = spawnSync(binary, args, { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  writeFileSync(`${capture}/${name}.stdout.raw`, result.stdout ?? "");
  writeFileSync(`${capture}/${name}.stderr.raw`, result.stderr ?? "");
  writeFileSync(
    `${capture}/${name}.process.json`,
    `${JSON.stringify(
      {
        source: process.env.SOURCE_SHA,
        executable: binary,
        executableSha256: binarySha256,
        argv: args,
        cwd: process.cwd(),
        pid: result.pid,
        status: result.status,
        signal: result.signal,
        error: result.error
          ? {
              name: result.error.name,
              message: result.error.message,
              stack: result.error.stack,
              code: result.error.code,
            }
          : null,
      },
      null,
      2,
    )}\n`,
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert.equal(hash(binary), binarySha256);
  return result;
};

const filter = "content_mapper_tsgo_cli::quoted_generic_parameters::";
const listed = run("inventory", [filter, "--list", "--format", "terse"]);
const actual = listed.stdout
  .split("\n")
  .filter((line) => line.endsWith(": test"))
  .map((line) => line.slice(0, -6))
  .sort();
const expected = [
  `${filter}quoted_generic_parameters_keep_complete_cli_and_official_host_packets_through_repairs`,
  `${filter}syntax::malformed_generic_attributes_keep_provider_syntax_codes_and_authored_ranges_through_repairs`,
  `${filter}body_error::independent_authored_body_errors_survive_generic_repairs`,
].sort();
assert.deepEqual(actual, expected, "the complete original three-test authority must execute");
writeFileSync(
  `${capture}/inventory.json`,
  `${JSON.stringify({ source: process.env.SOURCE_SHA, binary, binarySha256, actual }, null, 2)}\n`,
);

const executed = run("execution", [filter, "--test-threads=1", "--nocapture"]);
assert.match(executed.stdout, /test result: ok\. 3 passed; 0 failed; 0 ignored;/u);
// Read only this directly observed libtest PID's output. Cached or concurrent
// producers cannot enter the fresh execution inventory.
const root = path.join(
  "target/vize-tests/content-mapper-quoted-generic-parameters",
  String(executed.pid),
);
const schemas = new Map([
  ["vize.quoted-generic-parameters-host-observation", 32],
  ["vize.quoted-generic-syntax-host-observation", 48],
  ["vize.generic-body-error-host-observation", 64],
]);
const packets = [];
const walk = (directory) => {
  for (const item of readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, item.name);
    if (item.isDirectory()) {
      walk(file);
    } else if (item.isFile() && file.endsWith(".json")) {
      const bytes = readFileSync(file);
      const packet = JSON.parse(bytes);
      if (schemas.has(packet?.schema)) {
        packets.push({
          file,
          schema: packet.schema,
          sha256: createHash("sha256").update(bytes).digest("hex"),
        });
      }
    }
  }
};
walk(root);
for (const [schema, count] of schemas) {
  assert.equal(packets.filter((packet) => packet.schema === schema).length, count, schema);
}
assert.equal(packets.length, 144);
writeFileSync(
  `${capture}/whole-packet-inventory.json`,
  `${JSON.stringify(
    {
      source: process.env.SOURCE_SHA,
      executedPid: executed.pid,
      binarySha256,
      completePackets: packets.length,
      packets,
    },
    null,
    2,
  )}\n`,
);
