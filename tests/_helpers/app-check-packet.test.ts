import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { test } from "node:test";
import { retainAppCheckPacket } from "./app-check-packet.ts";

test("qualifier retention preserves the whole answer and physical config bytes before assertions", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-check-packet-"));
  const previous = process.env.VIZE_APP_CHECK_CAPTURE_DIR;
  try {
    const cwd = path.join(temporary, "project");
    fs.mkdirSync(cwd);
    const config =
      '{"extends":"./base.json","compilerOptions":{"baseUrl":".","types":[]}}\n// retained authored comment\n';
    const base = '{"compilerOptions":{"strict":true,"paths":{"@/*":["src/*"]}}}\n';
    fs.writeFileSync(path.join(cwd, "tsconfig.json"), config);
    fs.writeFileSync(path.join(cwd, "base.json"), base);
    fs.writeFileSync(
      path.join(cwd, "App.vue"),
      '<script setup lang="ts">const broken: number = "wrong";</script>\n',
    );
    const parsed = {
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions: { baseUrl: "." },
          files: ["App.vue"],
          extraProgramIdentity: "whole",
        },
      ],
      files: [
        {
          file: "App.vue",
          diagnostics: ["error:1:30 [TS2322] preserved"],
          extraSourceIdentity: "whole",
        },
      ],
      errorCount: 1,
      extraTopLevelIdentity: { unmodified: true },
    };
    const stdout = ` \n${JSON.stringify(parsed)}\n`;
    const input = {
      name: "fixture",
      stdout,
      parsed,
      cwd,
      command: "authored command",
      argv: ["vize", "check", "App.vue"],
    };
    const destination = path.join(temporary, "packets");
    delete process.env.VIZE_APP_CHECK_CAPTURE_DIR;
    retainAppCheckPacket(input);
    assert.equal(fs.existsSync(destination), false);
    process.env.VIZE_APP_CHECK_CAPTURE_DIR = destination;
    retainAppCheckPacket(input);
    const packet = path.join(destination, "fixture");
    assert.equal(fs.readFileSync(path.join(packet, "stdout.json"), "utf8"), stdout);
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(packet, "parsed.json"), "utf8")), parsed);
    const receipt = JSON.parse(fs.readFileSync(path.join(packet, "receipt.json"), "utf8"));
    assert.deepEqual(receipt.wholePrograms, parsed.programs);
    assert.deepEqual(receipt.authoredVizeArgv, input.argv);
    for (const [file, bytes] of [
      ["tsconfig.json", config],
      ["base.json", base],
    ]) {
      const record = receipt.files.find(
        (record: { path: string }) => record.path === path.join(cwd, file),
      );
      assert.equal(record.sha256, createHash("sha256").update(bytes).digest("hex"));
      assert.equal(fs.readFileSync(path.join(packet, record.retainedBytes), "utf8"), bytes);
    }
    assert.equal(receipt.observations[0].nativeSelectedConfigProved, false);
    assert.equal(
      receipt.missing.some(
        (record: { role: string }) => record.role === "physical-generated-config-candidate",
      ),
      true,
    );
  } finally {
    if (previous === undefined) delete process.env.VIZE_APP_CHECK_CAPTURE_DIR;
    else process.env.VIZE_APP_CHECK_CAPTURE_DIR = previous;
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
