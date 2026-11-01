import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import { sha256 } from "../../config/windows_consumer/common.ts";

export const macRole = "mac-arm64-node25-11264";
export const reportSha = "e40736f2951cab4ee3617aaa00930429adaeb287c826d7d9487a72015e02c8cd";
export const requiredFix = "729d3852a2200f45836571ec36056eeae1b7cd9b";
export type DispatchFields = {
  version?: string;
  producer?: string;
  authority?: string;
  beforeProducer?: string;
  beforeAuthority?: string;
};

/** Run in controls before routing either public job, including unknown-role failures. */
export function validateDispatch(
  event: string,
  ref: string,
  role: string,
  fields: DispatchFields,
): void {
  assert(["linux-node24", macRole].includes(role), "unknown formatter public role");
  if (event === "pull_request") return;
  assert.equal(event, "workflow_dispatch");
  assert.equal(ref, "refs/heads/main", "public consumers require main");
  if (role === macRole) {
    assert.equal(fields.version, "0.628.0");
    assert.match(fields.beforeProducer ?? "", /^[1-9]\d*$/u);
    assert.match(fields.beforeAuthority ?? "", /^[a-f0-9]{40}$/u);
  } else {
    assert.match(fields.version ?? "", /^\d+\.\d+\.\d+$/u);
  }
  assert.match(fields.producer ?? "", /^[1-9]\d*$/u);
  assert.match(fields.authority ?? "", /^[a-f0-9]{40}$/u);
}

export function macOriginal() {
  const value = asRecord(
    parseJson(
      readFileSync(
        new URL(
          "../../../../tests/tooling/fixtures/formatter-profiles-public-mac-original.json",
          import.meta.url,
        ),
        "utf8",
      ),
    ),
  );
  assert.equal(value.schema, "vize.formatter-profiles-public-mac-original.v1");
  assert.equal(value.issue, 11264);
  const raw = Buffer.from(asString(value.reportRawBase64), "base64");
  assert.equal(raw.toString("base64"), value.reportRawBase64);
  assert.equal(sha256(raw), reportSha);
  assert.equal(value.reportSha256, reportSha);
  const report = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  assert.equal(report.number, 11264);
  const body = asString(report.body);
  const source = body.split("```vue\n")[1]?.split("\n```")[0] + "\n";
  assert.equal(value.filename, "a.vue");
  assert.equal(value.source, source);
  assert.equal(value.sourceBytes, Buffer.byteLength(source));
  assert.equal(value.sourceSha256, sha256(source));
  assert.deepEqual(value.reportedEnvironment, {
    os: "macOS 26.6.2",
    platform: "darwin",
    arch: "arm64",
    node: "25.8.1",
    packageManager: "npm",
  });
  const equals = asArray(value.equals).map((value) => asString(value)),
    spaced = asArray(value.spaced).map((value) => asString(value));
  assert.deepEqual(equals, [
    "fmt",
    "a.vue",
    "--no-config",
    "--write",
    "--semi=false",
    "--bracket-spacing=false",
    "--sort-attributes=false",
  ]);
  assert.deepEqual(spaced, [
    "fmt",
    "a.vue",
    "--no-config",
    "--write",
    "--semi",
    "false",
    "--bracket-spacing",
    "false",
    "--sort-attributes",
    "false",
  ]);
  for (const args of [equals, spaced]) assert(body.includes("vize " + args.join(" ")));
  return { value, source, equals, spaced };
}

export function assertMacHost(platform: string, arch: string, node: string): void {
  assert.equal(platform, "darwin");
  assert.equal(arch, "arm64");
  assert.equal(node, "v25.8.1");
}

if (import.meta.main) {
  assert.equal(process.argv[2], "--validate-dispatch");
  validateDispatch(
    process.env.GITHUB_EVENT_NAME ?? "",
    process.env.GITHUB_REF ?? "",
    process.env.PUBLIC_ROLE || "linux-node24",
    {
      version: process.env.PUBLIC_VERSION,
      producer: process.env.PRODUCER_RUN,
      authority: process.env.AUTHORITY_BLOB,
      beforeProducer: process.env.BEFORE_PRODUCER_RUN,
      beforeAuthority: process.env.BEFORE_AUTHORITY_BLOB,
    },
  );
}
