import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asString } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { assertCustody, assertDeclarationCustody, sha256 } from "../computed_event_custody.ts";
import { assertProcess, assertRows, expected, modes, type Mode, type Role } from "./contract.ts";
import type { install } from "./install.ts";
import { capture } from "./process.ts";
import { query } from "./query.ts";

export function replaceOnce(source: string, before: string, after: string) {
  assert.equal(source.split(before).length, 2, "exactly one owned repair location");
  assert.notEqual(before, after);
  return source.replace(before, after);
}
export function inputs(mode: Mode) {
  const proof = readFileSync(
    fileURLToPath(
      new URL(
        "../../../../tests/fixtures/typechecker/public-slot-instance/proof.ts.txt",
        import.meta.url,
      ),
    ),
    "utf8",
  );
  assert.equal(sha256(proof), "1dae36a73d06b93dcff0696fac1165cd00622cfc7a5cac2c4a2f22e2f735d533");
  const preservation = readFileSync(
    fileURLToPath(new URL("./preservation.ts.txt", import.meta.url)),
    "utf8",
  );
  assert.equal(
    sha256(preservation),
    "925361fc4dacd625ca3c1bef8115a526fe2b66483a775c3eb07c1739a76f99b3",
  );
  const route = (source: string) =>
    mode === "direct"
      ? replaceOnce(source, '"vize/virtual-ts"', '"./node_modules/vize/virtual-ts.d.ts"')
      : source;
  return { target: route(proof), preservation: route(preservation) };
}
export function targetRepair(source: string) {
  return replaceOnce(
    source,
    "Result_DefaultZero.$slots.default({value:123});",
    "Result_DefaultZero.$slots.default({value:'repaired'});",
  );
}
export function preservationRepair(source: string) {
  return replaceOnce(
    source,
    'const wrong: number = "owned-negative";',
    "const wrong: number = 123;",
  );
}
export function observe(
  role: Role,
  installed: Awaited<ReturnType<typeof install>>,
  record: Recorder,
) {
  const observations = [];
  for (const mode of modes) {
    const project = join(record.root, "project-" + mode);
    mkdirSync(project);
    symlinkSync(installed.modules, join(project, "node_modules"), "dir");
    const packageBytes = JSON.stringify({ private: true, type: "module" });
    writeFileSync(join(project, "package.json"), packageBytes, { flag: "wx" });
    const file = "proof." + ({ esm: "mts", commonjs: "cts", direct: "ts" } as const)[mode],
      target = join(project, file);
    const config = JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "Node16",
        moduleResolution: "Node16",
        target: "ESNext",
        types: [],
      },
      files: [file],
    });
    writeFileSync(join(project, "tsconfig.json"), config, { flag: "wx" });
    const source = inputs(mode),
      cell = expected(role, mode);
    const before = {
      source,
      packageBytes,
      config,
      installedModuleRoot: realpathSync(join(project, "node_modules")),
    };
    record.json(mode + ".inputs.json", before);
    assert.equal(before.installedModuleRoot, installed.modules);
    const run = (label: string, sourceUtf8: string, stdout: string) => {
      writeFileSync(target, sourceUtf8);
      const input = {
        sourceUtf8,
        bytes: Buffer.byteLength(sourceUtf8),
        sha256: sha256(sourceUtf8),
      };
      record.json(mode + "-" + label + ".input.json", input);
      return [
        ["typescript", [process.execPath, installed.compiler]],
        ["native", [installed.checker]],
      ].map(([name, prefix]) => {
        assert.ok(Array.isArray(prefix));
        const packet = capture(
          record,
          mode + "-" + label + "-" + name,
          [...prefix, "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
          project,
        );
        assertProcess(packet, stdout);
        assert.equal(readFileSync(target, "utf8"), sourceUtf8);
        return { ...packet, input };
      });
    };
    const targetMessages = asArray(cell.targetMessages).map((value) => asString(value));
    assert.equal(targetMessages.length, 2);
    assert.equal(targetMessages[0], targetMessages[1]);
    const targetBad = run("target-bad", source.target, targetMessages[0]);
    const targetQuery = query(installed.ts, target, project, installed.root, false);
    record.json(mode + ".target-query.json", targetQuery);
    assertRows(role, mode, targetQuery, false);
    const targetFixed =
      role === "head" ? run("target-repair", targetRepair(source.target), "") : null;
    writeFileSync(target, source.target);
    assert.equal(readFileSync(target, "utf8"), source.target);
    const preserveMessages = asArray(cell.preservationMessages).map((value) => asString(value));
    assert.equal(preserveMessages.length, 2);
    assert.equal(preserveMessages[0], preserveMessages[1]);
    const preservationBad = run("preservation-bad", source.preservation, preserveMessages[0]);
    const preservationQuery = query(installed.ts, target, project, installed.root, true);
    record.json(mode + ".preservation-query.json", preservationQuery);
    assertRows(role, mode, preservationQuery, true);
    const preservationFixed = run(
      "preservation-repair",
      preservationRepair(source.preservation),
      "",
    );
    writeFileSync(target, source.target);
    assert.equal(readFileSync(target, "utf8"), source.target);
    assert.equal(readFileSync(join(project, "package.json"), "utf8"), packageBytes);
    assert.equal(readFileSync(join(project, "tsconfig.json"), "utf8"), config);
    assertCustody(
      [...targetQuery.inputs, ...preservationQuery.inputs].filter(
        (image) => image.path !== realpathSync(target),
      ),
    );
    assertCustody(installed.images);
    assertDeclarationCustody(installed.cliRoot, installed.graph.files, installed.graph.edges);
    observations.push({
      role,
      mode,
      targetBad,
      targetFixed,
      preservationBad,
      preservationFixed,
      targetQuery,
      preservationQuery,
    });
    record.json("complete-observations.json", observations);
  }
  return observations;
}
