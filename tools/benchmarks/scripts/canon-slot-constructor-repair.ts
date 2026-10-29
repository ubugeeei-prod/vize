/** Exact mapper/CLI admission for direct slot constructors and the reported real paths. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { slotCases } from "./canon-slot-constructor-cases.ts";
import { captureSlotPacket } from "./canon-slot-constructor-protocol.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";
import { packet, isRecord } from "./canon-instance-directives-model.ts";
import {
  authoredMappings,
  authoredSpan,
  diagnosticFingerprint,
  lines,
  occurrences,
  packetFiles,
  screenshotKey,
  virtualText,
} from "./canon-slot-constructor-real-project.ts";
import {
  auditPackets,
  cliContract,
  installedSdk,
  native,
  sdk,
  selectSdk,
  type Row,
} from "./canon-slot-constructor-audit.ts";

const realProjectFixture = path.resolve(
  "tests/_fixtures/differential/typechecker/zero-argument-slot-constructor-11013/real-project",
);
const realProjectFiles = [
  ["QSplitter.ts.txt", "src/components/QSplitter.ts"],
  ["SongEditor.vue.txt", "src/components/Song/SongEditor.vue"],
  ["ToolBar.vue.txt", "src/components/Song/ToolBar/ToolBar.vue"],
  ["tsconfig.json.txt", "tsconfig.json"],
] as const;

function captureRealProject(provider: string, head: string, checker: string, output: string) {
  const root = path.join(output, "real-project"),
    project = path.join(root, "project"),
    inputs = path.join(root, "inputs");
  fs.mkdirSync(project, { recursive: true });
  const sourceLines = new Map<string, number>(),
    inputIdentities: Record<string, ReturnType<typeof identify>> = {};
  for (const [fixtureName, projectName] of realProjectFiles) {
    const source = fs.readFileSync(path.join(realProjectFixture, fixtureName), "utf8"),
      projectFile = path.join(project, projectName),
      inputFile = path.join(inputs, projectName);
    fs.mkdirSync(path.dirname(projectFile), { recursive: true });
    fs.mkdirSync(path.dirname(inputFile), { recursive: true });
    fs.writeFileSync(projectFile, source);
    fs.writeFileSync(inputFile, source);
    inputIdentities[projectName] = identify(inputFile);
    if (projectName.endsWith(".vue")) sourceLines.set(projectName, lines(source));
  }
  const config: unknown = JSON.parse(fs.readFileSync(path.join(project, "tsconfig.json"), "utf8"));
  assert.ok(isRecord(config));
  save(path.join(project, "tsconfig.mapper.json"), {
    ...config,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
  fs.copyFileSync(
    path.join(project, "tsconfig.mapper.json"),
    path.join(inputs, "tsconfig.mapper.json"),
  );
  inputIdentities["tsconfig.mapper.json"] = identify(path.join(inputs, "tsconfig.mapper.json"));
  const provenanceFile = path.join(realProjectFixture, "screenshot-provenance.json"),
    provenanceBytes = fs.readFileSync(provenanceFile),
    provenance: unknown = JSON.parse(provenanceBytes.toString("utf8"));
  fs.writeFileSync(path.join(inputs, "screenshot-provenance.json"), provenanceBytes);
  inputIdentities["screenshot-provenance.json"] = identify(
    path.join(inputs, "screenshot-provenance.json"),
  );
  assert.ok(isRecord(provenance) && Array.isArray(provenance.diagnostics));
  const screenshotDiagnostics = provenance.diagnostics.map((value: unknown) => {
    assert.ok(isRecord(value));
    const { file, line, column, code, message, location } = value;
    assert.ok(
      typeof file === "string" &&
        typeof line === "number" &&
        typeof column === "number" &&
        typeof code === "number" &&
        typeof message === "string" &&
        location === "virtual-only",
    );
    return { file, line, column, code, message, location };
  });
  assert.equal(provenance.kind, "user-supplied-v0.603.0-screenshot");
  installedSdk(project, provider);
  const packageFile = path.join(project, "node_modules/vize/package.json"),
    publishedManifest: unknown = JSON.parse(fs.readFileSync(packageFile, "utf8"));
  assert.ok(isRecord(publishedManifest));
  assert.equal(publishedManifest.version, "0.603.0");
  fs.copyFileSync(packageFile, path.join(inputs, "published-vize-package.json"));
  const publishedPackageIdentity = identify(path.join(inputs, "published-vize-package.json"));
  inputIdentities["published-vize-package.json"] = publishedPackageIdentity;
  const publishedBinary = path.join(project, "node_modules/vize/bin/vize"),
    packetRecords: unknown[] = [],
    publishedPackets = new Map<string, Record<string, unknown>>();
  for (const [, projectName] of realProjectFiles.filter(([, name]) => name.endsWith(".vue"))) {
    const source = fs.readFileSync(path.join(project, projectName), "utf8"),
      packetName = path.basename(projectName, ".vue"),
      publishedRoot = path.join(root, "packets/published-v0.603.0", packetName),
      headRoot = path.join(root, "packets/head", packetName),
      publishedPacket = captureSlotPacket(
        publishedBinary,
        project,
        source,
        publishedRoot,
        projectName,
      ),
      headPacket = captureSlotPacket(head, project, source, headRoot, projectName),
      span = authoredSpan(source),
      packetAudit = auditPackets(publishedPacket.result, headPacket.result, "QSplitter"),
      publishedText = virtualText(publishedPacket.result),
      headText = virtualText(headPacket.result),
      legacyCall = "new (undefined as unknown as __VizeSlotConstructor<typeof QSplitter>)({";
    assert.equal(occurrences(publishedText, legacyCall), 2);
    assert.equal(occurrences(headText, legacyCall), 0);
    const publishedAuthoredMappings = authoredMappings(publishedPacket.result, span),
      headAuthoredMappings = authoredMappings(headPacket.result, span);
    assert.deepEqual(headAuthoredMappings, publishedAuthoredMappings);
    publishedPackets.set(projectName, publishedPacket.result);
    packetRecords.push({
      file: projectName,
      authoredComponentSpan: span,
      source: inputIdentities[projectName],
      published: {
        files: packetFiles(publishedRoot, publishedPacket),
        legacyConstructorCalls: 2,
        authoredMappings: publishedAuthoredMappings,
      },
      head: {
        files: packetFiles(headRoot, headPacket),
        directConstructorBranches: packetAudit.directConstructorBranches,
        authoredMappings: headAuthoredMappings,
      },
      packetAudit,
    });
  }
  const args = [
      "--runExternalCode",
      "--checkers",
      "1",
      "--noEmit",
      "--pretty",
      "false",
      "--project",
      "tsconfig.mapper.json",
    ],
    execute = (role: string) => {
      const destination = path.join(root, role),
        env = {
          ...process.env,
          TMPDIR: path.join(destination, "tmp"),
          XDG_CACHE_HOME: path.join(destination, "cache"),
        };
      fs.mkdirSync(env.TMPDIR, { recursive: true });
      const result = run(destination, "mapper", checker, args, project, env);
      return native(result.stdout, result.status, project);
    },
    baseline = execute("published-v0.603.0");
  assert.ok(baseline.length > 0, "the published project must reproduce the screenshot failure");
  for (const diagnostic of baseline) {
    assert.equal(diagnostic.code, 2554);
    assert.equal(diagnostic.message, "Expected 0 arguments, but got 1.");
    const sourceLineCount = sourceLines.get(diagnostic.file);
    assert.ok(sourceLineCount !== undefined, `unexpected diagnostic file ${diagnostic.file}`);
    assert.ok(
      diagnostic.line > sourceLineCount,
      `published ${diagnostic.file} diagnostic must be virtual-only`,
    );
  }
  const publishedFingerprints = baseline.map((diagnostic) => {
    const packet = publishedPackets.get(diagnostic.file),
      sourceLineCount = sourceLines.get(diagnostic.file);
    assert.ok(packet && sourceLineCount !== undefined);
    return diagnosticFingerprint(diagnostic, packet, sourceLineCount);
  });
  const expectedScreenshotMultiset = screenshotDiagnostics.map(screenshotKey).sort(),
    observedScreenshotDiagnostics = [
      ...new Map(
        baseline.map((diagnostic) => [
          screenshotKey(diagnostic),
          {
            file: diagnostic.file,
            code: diagnostic.code,
            message: diagnostic.message,
          },
        ]),
      ).values(),
    ].sort((left, right) => left.file.localeCompare(right.file)),
    observedScreenshotMultiset = observedScreenshotDiagnostics.map(screenshotKey).sort();
  assert.deepEqual(
    observedScreenshotMultiset,
    expectedScreenshotMultiset,
    "the published Corsa surface must reproduce both exact screenshot paths and TS2554 messages",
  );
  selectSdk(project, head);
  const repaired = execute("head");
  assert.deepEqual(repaired, [], "the repaired real project must have no virtual-only diagnostics");
  const receipt = {
    id: "real-v0.603.0-qsplitter-screenshot",
    passed: true,
    timingUsable: false,
    providerLock: identify(path.join(provider, "package-lock.json")),
    publishedPackage: {
      manifest: publishedPackageIdentity,
      version: publishedManifest.version,
    },
    head: identify(head),
    checker: identify(checker),
    provenance,
    inputs: inputIdentities,
    sourceLines: Object.fromEntries(sourceLines),
    published: {
      rawDiagnostics: baseline,
      exactScreenshotMultiset: observedScreenshotDiagnostics,
      inferenceEchoMultiplicity: observedScreenshotDiagnostics.map((diagnostic) => ({
        ...diagnostic,
        count: baseline.filter((row) => screenshotKey(row) === screenshotKey(diagnostic)).length,
      })),
      virtualOriginFingerprints: publishedFingerprints,
      virtualOnly: publishedFingerprints.every((entry) => entry.virtualOnly),
    },
    repaired: {
      rawDiagnostics: repaired,
      exactTs2554Multiset: [],
      virtualOnlyDiagnostics: [],
    },
    packets: packetRecords,
    scope:
      "real-path Corsa semantic oracle; published and head binaries differ; no timing comparison",
  };
  save(path.join(root, "acceptance.json"), receipt);
  return receipt;
}

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
  let realProject: unknown = null;
  try {
    realProject = captureRealProject(provider, head, checker, output);
  } catch (error) {
    const failure = {
      id: "real-v0.603.0-qsplitter-screenshot",
      error: error instanceof Error ? (error.stack ?? error.message) : String(error),
    };
    failures.push(failure);
    save(path.join(output, "real-project/failure.json"), failure);
  }
  assert.deepEqual(identify(path.join(provider, "package-lock.json")), providerBefore);
  save(path.join(output, "acceptance.json"), {
    entries,
    realProject,
    failures,
    passed: failures.length === 0,
    timingUsable: false,
  });
  assert.equal(failures.length, 0, "no constructor argument or slot precision failure may survive");
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
