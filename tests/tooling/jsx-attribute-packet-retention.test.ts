import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

const rust = fs.readFileSync(
  new URL(
    "../../crates/vize/tests/check_jsx_slot_parameter_types_cli/attribute_expressions.rs",
    import.meta.url,
  ),
  "utf8",
);
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/pr-rust-products.yml", import.meta.url),
  "utf8",
);
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
function section(text: string, from: string, to: string) {
  const start = text.indexOf(from),
    end = text.indexOf(to, start + from.length);
  assert(start >= 0 && end > start);
  return text.slice(start, end);
}
function originalLaw(text: string) {
  for (const [from, to, bytes, sha] of [
    [
      "        let member = if input ==",
      "\nfn executable_hash",
      1780,
      "395c82db03f1aeb703839f49b8f9761d378fc249f962ac16588b0c8027917f0d",
    ],
    [
      "        let mut command = Command::new",
      "        let arguments: Vec<_>",
      336,
      "39fefc185fe56de4bf25752c8bc1a145430048fdcd887cca3b209addd08d96bc",
    ],
    [
      "        let host = std::fs::read",
      "        let mut command = Command::new",
      1238,
      "99f87034c23d1de2a7c6795f07186298732340819b43f1784c1f39c0cf482178",
    ],
  ] as const) {
    const body = section(text, from, to);
    assert.equal(Buffer.byteLength(body), bytes);
    assert.equal(digest(body), sha);
  }
  assert(
    text.includes(`        ("valid", "valid", false, true),
        ("member", "invalid", false, true),
        ("semantic-options", "valid", true, true),
        ("semantic-options-explicit-false", "valid", true, false),`),
  );
}
function failureFirst(text: string) {
  const complete = section(
    text,
    '        process["exitCode"] = json!(result.status.code());',
    "        let member = if input ==",
  );
  const saved = complete.indexOf('"completed"');
  assert(saved >= 0 && saved < complete.indexOf('project.join("stdout.raw")'));
  assert(complete.includes("Some(&result)"));
  assert(complete.includes('project.join("stderr.raw")'));
  assert(complete.includes('project.join("process.json")'));
  for (const [from, to, stage] of [
    ["                    Err(error) => {", "            process[field]", "field"],
    [
      "        let child = command.spawn().unwrap_or_else",
      "        let pid = child.id();",
      '"spawn"',
    ],
    [
      "        let result = child.wait_with_output().unwrap_or_else",
      '        process["exitCode"]',
      '"wait"',
    ],
  ]) {
    const handler = section(text, from, to);
    assert(handler.includes("retain_packet("));
    assert(handler.includes("panic!("));
    assert(handler.indexOf("retain_packet(") < handler.indexOf("panic!("));
    assert(handler.includes(stage));
    assert(handler.includes("Some(json!(error.to_string()))"));
    assert(handler.includes("None,"));
  }
  assert(text.includes('process["pid"] = json!(pid);'));
  assert(text.includes('"nativeChildPid":null,"nativeImageLoaded":"unobserved"'));
}
function closedPacket(text: string) {
  const inputs = section(text, "fn owned_inputs(", "\n// Data only,");
  const names = section(inputs, "    [", "    ]").match(/"[^"]+"/gu);
  assert.deepEqual(names, [
    '"Host.vue"',
    '"Consumer.tsx"',
    '"Probe.vue"',
    '"tsconfig.json"',
    '"vite.config.mjs"',
  ]);
  assert(inputs.includes('"state":"absent","data":null'));
  assert(inputs.includes('"state":"unavailable","data":null,"error":'));
  assert(inputs.includes("std::fs::symlink_metadata(project.join(name))"));
  assert(inputs.includes("!metadata.file_type().is_file()"));
  assert(!/read_dir|walk|copy|(?:fs::)?symlink\(/iu.test(inputs));
  const packet = text.slice(text.indexOf("fn retain_packet("));
  for (const required of [
    '"schema":"vize.jsx-attribute-native-cli-packet","version":1',
    '"inputsBefore":inputs_before,"inputsAfter":owned_inputs(project)',
    '"stdout":result.map(|output| data_bytes(&output.stdout))',
    '"stderr":result.map(|output| data_bytes(&output.stderr))',
    '"status":result.map(|output| json!',
    '"success":output.status.success(),"signal":signal',
    '.open(packets.join(format!("{id}.json")))',
    ".create_new(true)",
    "file.sync_all().unwrap();",
  ])
    assert(packet.includes(required), required);
  assert(!/read_dir|copy|symlink|unwrap_or_default/iu.test(packet));
  assert(text.includes('json!({"bytes":bytes,"length":bytes.len(),"sha256":digest})'));
}
function packetRoute(text: string) {
  const tc = section(text, '          if [[ "$COHORT" == "typechecker" ]]; then', "          fi");
  assert.equal((text.match(/VIZE_JSX_ATTRIBUTE_PACKET_DIR/gu) ?? []).length, 1);
  assert(tc.includes("printf 'VIZE_JSX_ATTRIBUTE_PACKET_DIR=%s\\n'"));
  assert(
    tc.includes(
      '"$GITHUB_WORKSPACE/target/nextest/$NEXTEST_PROFILE/jsx-attribute-packets/$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$GITHUB_SHA"',
    ),
  );
  assert(text.includes("path: target/nextest/${{ env.NEXTEST_PROFILE }}/"));
  assert(!tc.includes("VIZE_JSX_SLOT_CAPTURE"));
}

test("original four cases, inputs, child argv/stdio and complete oracles are byte-preserved", () => {
  originalLaw(rust);
  for (const mutant of [
    rust.replace("Some(i32::from(errors > 0))", "Some(2)"),
    rust.replace('"--quiet", "--format"', '"--format"'),
    rust.replace('"target":"ES2022"', '"target":"ESNext"'),
    rust.replace('("member", "invalid", false, true)', '("member", "valid", false, true)'),
  ])
    assert.throws(() => originalLaw(mutant));
});
test("completed and hash/spawn/wait errors retain packets before every outcome assertion", () => {
  failureFirst(rust);
  for (const mutant of [
    rust.replace('"completed"', '"discarded"'),
    rust.replace(
      "let child = command.spawn().unwrap_or_else",
      "let child = command.spawn().unwrap(); //",
    ),
    rust.replace("Some(json!(error.to_string()))", "None"),
    rust.replace('process["pid"] = json!(pid);', 'process["pid"] = json!(0);'),
    rust.replace('"nativeChildPid":null', '"nativeChildPid":pid'),
  ])
    assert.throws(() => failureFirst(mutant));
});
test("closed data-only byte records preserve missing streams and explicit owned-input absences", () => {
  closedPacket(rust);
  for (const mutant of [
    rust.replace(
      '"Probe.vue",\n        "tsconfig.json",',
      '"node_modules",\n        "tsconfig.json",',
    ),
    rust.replace(
      '"stdout":result.map(|output| data_bytes(&output.stdout))',
      '"stdout":data_bytes(&[])',
    ),
    rust.replace(".create_new(true)", ".create_new(false)"),
    rust.replace("!metadata.file_type().is_file()", "false"),
    rust.replace('"state":"absent","data":null', '"state":"present","data":[]'),
  ])
    assert.throws(() => closedPacket(mutant));
});
test("only the typechecker routes run-bound JSON leaves into its existing artifact", () => {
  packetRoute(workflow);
  for (const mutant of [
    workflow.replace("VIZE_JSX_ATTRIBUTE_PACKET_DIR", "VIZE_JSX_SLOT_CAPTURE"),
    workflow.replace("$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$GITHUB_SHA", "stale"),
    workflow.replace(
      "path: target/nextest/${{ env.NEXTEST_PROFILE }}/",
      "path: target/vize-tests/",
    ),
  ])
    assert.throws(() => packetRoute(mutant));
});
