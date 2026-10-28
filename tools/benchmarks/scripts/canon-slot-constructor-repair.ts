/** Exact mapper/CLI admission for slot-constructor argument tuples. This is not a speed benchmark. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { slotCases } from "./canon-slot-constructor-cases.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";
import { packet, isRecord } from "./canon-instance-directives-model.ts";
import {
  auditPackets,
  cliContract,
  native,
  sdk,
  selectSdk,
  type Row,
} from "./canon-slot-constructor-audit.ts";
async function main() {
  const [providerArg, baseArg, headArg, outputArg] = process.argv.slice(2);
  assert.ok(providerArg && baseArg && headArg && outputArg);
  const provider = path.resolve(providerArg),
    base = path.resolve(baseArg),
    head = path.resolve(headArg),
    output = path.resolve(outputArg);
  fs.mkdirSync(output);
  const getter: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  assert.ok(isRecord(getter) && typeof getter.default === "function");
  const checker: unknown = Reflect.apply(getter.default, getter, []);
  assert.ok(typeof checker === "string");
  const providerBefore = identify(path.join(provider, "package-lock.json")),
    entries: unknown[] = [],
    failures: unknown[] = [];
  save(path.join(output, "images.json"), {
    base: identify(base),
    head: identify(head),
    checker: identify(checker),
    providerLock: providerBefore,
    scope:
      "semantic admission; base shipping and head ci-opt profiles differ; no timing comparison",
  });
  assert.notEqual(identify(base).sha256, identify(head).sha256);
  for (const row of slotCases()) {
    const root = path.join(output, row.id),
      project = path.join(root, "project");
    fs.mkdirSync(path.join(project, "src"), { recursive: true });
    fs.writeFileSync(path.join(project, "component.ts"), row.component);
    fs.writeFileSync(path.join(project, "src/Tiny.vue"), row.source);
    const config = {
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ES2024",
        lib: ["ES2024", "DOM"],
        types: [],
        incremental: false,
      },
      vueCompilerOptions: { strictTemplates: true, checkUnknownProps: true },
      include: ["src/**/*.vue", "component.ts"],
    };
    save(path.join(project, "tsconfig.json"), config);
    save(path.join(project, "tsconfig.mapper.json"), {
      ...config,
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    });
    sdk(project, provider, base);
    for (const file of ["component.ts", "src/Tiny.vue", "tsconfig.json", "tsconfig.mapper.json"]) {
      const dest = path.join(root, "inputs", file);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(path.join(project, file), dest);
    }
    try {
      const basePacket = captureSlotPacket(
        base,
        project,
        row.source,
        path.join(root, "base-packet"),
      );
      const headPacket = captureSlotPacket(
        head,
        project,
        row.source,
        path.join(root, "head-packet"),
      );
      const packetAudit = auditPackets(basePacket.result, headPacket.result);
      const observed = new Map<string, { native: Row[]; cli: ReturnType<typeof packet> }>();
      for (const [role, binary] of [
        ["base", base],
        ["head", head],
      ]) {
        assert.ok(role && binary);
        selectSdk(project, binary);
        const out = path.join(root, role),
          env = {
            ...process.env,
            TMPDIR: path.join(out, "tmp"),
            XDG_CACHE_HOME: path.join(out, "cache"),
          };
        fs.mkdirSync(env.TMPDIR, { recursive: true });
        const nr = run(
          out,
          "mapper",
          checker,
          [
            "--runExternalCode",
            "--checkers",
            "1",
            "--noEmit",
            "--pretty",
            "false",
            "--project",
            "tsconfig.mapper.json",
          ],
          project,
          env,
        );
        const cr = run(
          out,
          "cli",
          binary,
          [
            "check",
            "--no-config",
            "--format",
            "json",
            "--show-virtual-ts",
            "--tsconfig",
            "tsconfig.json",
            "--corsa-path",
            checker,
          ],
          project,
          env,
        );
        observed.set(role, {
          native: native(nr.stdout, nr.status, project),
          cli: packet(cr.stdout, cr.status, project),
        });
      }
      const b = observed.get("base"),
        h = observed.get("head");
      assert.ok(b && h);
      const sourceLines = row.source.split(/\r\n|[\r\n\u2028\u2029]/u).length,
        authored = (rows: Row[]) => rows.filter((diagnostic) => diagnostic.line <= sourceLines),
        generated = (rows: Row[]) => rows.filter((diagnostic) => diagnostic.line > sourceLines);
      assert.deepEqual(
        authored(h.native),
        authored(b.native),
        "every authored native diagnostic stays exact",
      );
      assert.ok(
        h.native.every((diagnostic) => diagnostic.code !== 2554),
        "the repaired constructor call must not report TS2554",
      );
      if (row.id === "zero") {
        assert.deepEqual(
          generated(b.native).map((diagnostic) => diagnostic.code),
          [2554, 2554],
          "the shipping mapper must reproduce both generated zero-argument failures",
        );
        assert.deepEqual(h.native, [], "the exact valid reporter path must be clean after repair");
      }
      const headGenerated = generated(h.native);
      if (row.id === "constructor-public-props-discrepancy") {
        assert.equal(
          generated(b.native).length,
          2,
          "both baseline inference phases must reject it",
        );
        assert.equal(
          headGenerated.length,
          2,
          "both repaired inference phases must still reject it",
        );
        for (const diagnostic of headGenerated) {
          assert.equal(diagnostic.code, 2741);
          assert.match(diagnostic.message, /^Property 'internal' is missing /u);
          assert.match(diagnostic.message, /internal: string/u);
        }
      } else {
        assert.deepEqual(headGenerated, [], "the repair must add no generated diagnostics");
      }
      assert.deepEqual(
        cliContract(h.cli),
        cliContract(b.cli),
        "whole public CLI diagnostics and project metadata stay exact",
      );
      if (row.requiresAuthoredError) {
        assert.ok(
          h.native.some((d) => d.line <= row.source.split("\n").length),
          "independent authored invalid mutation must diagnose",
        );
        const original = slotCases().find((item) => item.id === "zero");
        assert.ok(original);
        fs.writeFileSync(path.join(project, "component.ts"), original.component);
        fs.writeFileSync(path.join(project, "src/Tiny.vue"), original.source);
        selectSdk(project, head);
        const repaired = run(
          path.join(root, "repair"),
          "mapper",
          checker,
          [
            "--runExternalCode",
            "--checkers",
            "1",
            "--noEmit",
            "--pretty",
            "false",
            "--project",
            "tsconfig.mapper.json",
          ],
          project,
        );
        assert.deepEqual(
          native(repaired.stdout, repaired.status, project),
          [],
          "repair restores the independent original valid source",
        );
      }
      const entry = {
        id: row.id,
        passed: true,
        sourceSha256: hash(row.source),
        componentSha256: hash(row.component),
        packetAudit,
        baseline: b.native,
        head: h.native,
        timingUsable: false,
      };
      entries.push(entry);
      save(path.join(root, "acceptance.json"), entry);
    } catch (error) {
      const failure = {
        id: row.id,
        error: error instanceof Error ? (error.stack ?? error.message) : String(error),
      };
      failures.push(failure);
      save(path.join(root, "failure.json"), failure);
    }
  }
  assert.deepEqual(identify(path.join(provider, "package-lock.json")), providerBefore);
  save(path.join(output, "acceptance.json"), {
    entries,
    failures,
    passed: failures.length === 0,
    timingUsable: false,
  });
  assert.equal(failures.length, 0, "no constructor argument or slot precision failure may survive");
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
