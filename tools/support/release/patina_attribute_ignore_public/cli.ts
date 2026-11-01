import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { parseJson } from "../../json-value.ts";
import { hook, observed, verifyJournal } from "../editor_vite_stub/custody.ts";
import { preserve, type Context } from "./bootstrap.ts";
import { cleanCli, corpus, expectedCli, referenceFor, rule, sourceFor } from "./contract.ts";

export function configSource(policy: unknown, dynamic: boolean, disabled = false): string {
  const native = {
    linter: {
      preset: "incremental",
      rules: { [rule]: disabled ? "off" : "error" },
      ruleOptions: { [rule]: policy },
    },
  };
  const literal = `{ vue: ${JSON.stringify(native)} }`;
  return `export default ${dynamic ? "() => (" + literal + ")" : literal};\n`;
}
export function runCli(context: Context): void {
  mkdirSync(join(context.root, "src"));
  const input = join(context.root, "src/Probe.vue"),
    config = join(context.root, "vite.config.mjs");
  let calls = 0;
  for (const dynamic of [false, true])
    for (const row of corpus.valid) {
      const source = sourceFor(row),
        reference = referenceFor(row),
        carrier = configSource(row.policy, dynamic);
      writeFileSync(input, source);
      writeFileSync(config, carrier);
      for (const pass of ["verify", "fix", "requery"]) {
        const label = `cli-${row.id}-${dynamic ? "function" : "literal"}-${pass}`;
        const before = readFileSync(input, "utf8");
        context.record.json(label + ".input.json", {
          id: row.id,
          dynamic,
          pass,
          source,
          config: carrier,
          before,
        });
        const audit = observed(context.root, context.identity, context.record, label);
        const argv = [
          process.execPath,
          "--require",
          hook,
          context.cli,
          "lint",
          "--config",
          "vite.config.mjs",
          "--format",
          "json",
          "src/Probe.vue",
          ...(pass === "fix" ? ["--fix"] : []),
        ];
        const packet = context.record.run(label, argv, context.root, audit.env);
        const after = readFileSync(input, "utf8");
        context.record.json(label + ".observation.json", {
          id: row.id,
          dynamic,
          pass,
          source,
          config: carrier,
          before,
          after,
          packet,
          journal: audit.journal,
        });
        calls++;
        assert.equal(packet.error, null);
        assert.equal(packet.signal, null);
        assert.equal(packet.status, pass === "verify" && row.expectedCount > 0 ? 1 : 0, row.id);
        assert.deepEqual(
          parseJson(packet.stdout),
          pass === "verify" ? expectedCli(row) : cleanCli(),
          row.id,
        );
        assert.equal(after, pass === "verify" ? source : reference.fixedSource, row.id);
        assert.equal(readFileSync(config, "utf8"), carrier);
        verifyJournal(audit.journal, context.identity, packet.status);
        preserve(context);
      }
    }
  const row = corpus.valid[0]!,
    source = sourceFor(row),
    carrier = configSource({ ignore: [] }, false, true);
  writeFileSync(input, source);
  writeFileSync(config, carrier);
  const label = "cli-original-disabled-control",
    audit = observed(context.root, context.identity, context.record, label);
  const packet = context.record.run(
    label,
    [
      process.execPath,
      "--require",
      hook,
      context.cli,
      "lint",
      "--config",
      "vite.config.mjs",
      "--format",
      "json",
      "--fix",
      "src/Probe.vue",
    ],
    context.root,
    audit.env,
  );
  const after = readFileSync(input, "utf8");
  context.record.json(label + ".observation.json", {
    source,
    config: carrier,
    packet,
    after,
    journal: audit.journal,
  });
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.status, 0);
  assert.deepEqual(parseJson(packet.stdout), cleanCli());
  assert.equal(after, source);
  assert.equal(readFileSync(config, "utf8"), carrier);
  verifyJournal(audit.journal, context.identity, packet.status);
  preserve(context);
  calls++;
  assert.equal(calls, 55);
  context.record.json("cli-complete.json", {
    complete: true,
    inputs: 9,
    carriers: 2,
    calls,
    limits:
      "Whole original stdout JSON, process statuses, actual file fixes and fixed points. Unmodified stderr is retained in every packet; no cross-rule or performance claim.",
  });
}
