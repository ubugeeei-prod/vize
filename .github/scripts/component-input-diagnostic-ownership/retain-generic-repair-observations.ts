import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";

const capture = `${process.env.RUNNER_TEMP}/component-input-ownership/generic-regression`;
interface CargoMessage {
  reason: string;
  success?: boolean;
  target?: { kind: string[]; name: string };
  executable?: string;
}

const messages = readFileSync(`${capture}/build.jsonl`, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line) as CargoMessage);
const finalMessage = messages.at(-1);
assert.ok(finalMessage);
assert.equal(finalMessage.reason, "build-finished");
assert.equal(finalMessage.success, true);

const targets = messages.filter(
  (message): message is CargoMessage & { executable: string } =>
    message.reason === "compiler-artifact" &&
    message.target?.name === "integration" &&
    message.target?.kind.join() === "test" &&
    typeof message.executable === "string",
);
assert.equal(targets.length, 1);

const binary = realpathSync(targets[0].executable);
const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const binarySha256 = hash(binary);
const run = (name: string, args: string[]): SpawnSyncReturns<string> => {
  const result = spawnSync(binary, args, { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  const error = result.error as (Error & { code?: string }) | undefined;
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
        error: error
          ? {
              name: error.name,
              message: error.message,
              stack: error.stack,
              code: error.code,
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
const packets: Array<{ file: string; schema: string; sha256: string }> = [];
const walk = (directory: string): void => {
  for (const item of readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, item.name);
    if (item.isDirectory()) {
      walk(file);
    } else if (item.isFile() && file.endsWith(".json")) {
      const bytes = readFileSync(file);
      const packet = JSON.parse(bytes.toString()) as { schema?: string };
      if (packet.schema && schemas.has(packet.schema)) {
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
