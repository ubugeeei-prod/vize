import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  decodeFrames,
  diagnostics,
  metrics,
  resourceUsage,
  deriveConfig,
  installedNative,
  inventory,
} from "./template-alias-paired-11822-custody.mjs";

test("derivative configs preserve every original scope and mapper option", () => {
  const original = {
    extends: "./tsconfig.base.json",
    files: ["src/main.ts"],
    include: ["src/**/*.vue"],
    exclude: ["ignored"],
    compilerOptions: { strict: true, types: [], incremental: true },
    contentMappers: [
      {
        package: "@vue/content-mapper",
        extensions: [".vue", ".custom"],
        options: { strictTemplates: true },
      },
    ],
  };
  const before = structuredClone(original);
  const derived = deriveConfig(original, "@vue/content-mapper");
  assert.deepEqual(original, before);
  assert.deepEqual(derived, {
    ...before,
    compilerOptions: { ...before.compilerOptions, incremental: false },
    contentMappers: [{ ...before.contentMappers[0], package: "template-alias-vue" }],
  });
  assert.throws(() => deriveConfig(original, "vize"));
});

test("native custody requires exact package version, SRI and real Linux x64 ELF", (t) => {
  const root = mkdtempSync(join(tmpdir(), "template-alias-custody-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const packageRoot = join(root, "node_modules/@tsrs/linux-x64-gnu");
  mkdirSync(packageRoot, { recursive: true });
  writeFileSync(
    join(packageRoot, "package.json"),
    JSON.stringify({ name: "@tsrs/linux-x64-gnu", version: "0.21.0-ts7.1.0-beta" }),
  );
  writeFileSync(
    join(root, "pnpm-lock.yaml"),
    "  '@tsrs/linux-x64-gnu@0.21.0-ts7.1.0-beta':\n    resolution: {integrity: sha512-example}\n",
  );
  const elf = Buffer.alloc(20);
  Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]).copy(elf);
  elf.writeUInt16LE(62, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  const native = () =>
    installedNative(root, inventory(root), "@tsrs/linux-x64-gnu", "0.21.0-ts7.1.0-beta", ["tsrs"]);
  assert.equal(native().integrity, "sha512-example");
  elf.writeUInt16LE(183, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  assert.throws(native, /x86-64/);
  elf.writeUInt16LE(62, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  writeFileSync(join(root, "pnpm-lock.yaml"), "");
  assert.throws(native, /absent from pinned lock/);
});

const expected = Array.from({ length: 12 }, (_, n) => ({
  file: `src/generated/GeneratedComponent${String(n).padStart(4, "0")}.vue`,
  line: 40,
  column: n + 1,
  code: 2322,
}));
const output = expected
  .map(
    (d) =>
      `/work/${d.file}(${d.line},${d.column}): error TS${d.code}: Wrong value ${d.column}.\n  Detail ${d.column}.`,
  )
  .join("\n");

test("parity retains complete multiline messages and rejects duplicate, extra and unparsed errors", () => {
  const packet = diagnostics(output, "/work", expected);
  assert.equal(packet.length, 12);
  assert.equal(packet[0].message, "Wrong value 1.\n  Detail 1.");
  assert.throws(
    () => diagnostics(output + "\n" + output.split("\n")[0], "/work", expected),
    /twelve/,
  );
  assert.throws(
    () => diagnostics(output + "\nerror TS6142: Missing JSX mode", "/work", expected),
    /unparsed/,
  );
  assert.throws(
    () => diagnostics(output.replace("TS2322", "TS2345"), "/work", expected),
    /locations/,
  );
  assert.throws(
    () => diagnostics(output.split("\n").slice(2).join("\n"), "/work", expected),
    /twelve/,
  );
  assert.notDeepEqual(
    packet,
    diagnostics(output.replace("Wrong value 1.", "Changed message."), "/work", expected),
  );
});

test("protocol custody retains Unicode, directives and all bytes, refusing incomplete or ambiguous frames", () => {
  const message = {
    id: 1,
    result: {
      text: "const name = '日本語';",
      diagnosticDirectives: { directives: [[0, 1, 2, 3]] },
    },
  };
  const body = Buffer.from(JSON.stringify(message));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  assert.deepEqual(decodeFrames(Buffer.concat([frame, frame])), [message, message]);
  assert.throws(() => decodeFrames(frame.subarray(0, -1)), /truncated/);
  assert.throws(
    () => decodeFrames(Buffer.from("Content-Length: 1\r\nContent-Length: 1\r\n\r\n0")),
    /ambiguous/,
  );
  assert.throws(() => decodeFrames(Buffer.concat([frame, Buffer.from("garbage")])), /truncated/);
});

test("phase and resource receipts cannot pass with incomplete or early-failure output", () => {
  const text =
    "Content mapper request wait time: 0.392s\nCheck time: 2.478s\nTypes: 4684959\nInstantiations: 49119876\nMemory used: 2296581K\n";
  assert.deepEqual(metrics(text), {
    mapperWaitSeconds: 0.392,
    checkSeconds: 2.478,
    types: 4684959,
    instantiations: 49119876,
    checkerMemoryKiB: 2296581,
  });
  assert.throws(() => metrics(text.replace("Check time: 2.478s\n", "")), /Check time/);
  assert.deepEqual(
    resourceUsage(
      "Command exited with non-zero status 2\nwall_s=3.1\nuser_s=10.2\nsystem_s=1.0\nmax_rss_kib=123456\nstatus=2\n",
    ),
    { wall_s: 3.1, user_s: 10.2, system_s: 1, max_rss_kib: 123456, status: 2 },
  );
  assert.throws(() => resourceUsage("wall_s=0.1"), /user_s/);
});
