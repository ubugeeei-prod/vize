import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runtime } from "./identity.ts";
import {
  admissionFiles,
  globalRefAdmission,
  mutatedAdmission,
  verifyOriginals,
  baseProject,
  globalProject,
} from "./fixtures.ts";
import { checkProject } from "./process.ts";
import { lspAdmission } from "./lsp-probes.ts";
import { hash, write } from "./model.ts";
async function main(): Promise<void> {
  const r = await runtime(process.argv.slice(2));
  const capsule = path.dirname(fileURLToPath(import.meta.url));
  const originals = verifyOriginals(capsule);
  write(path.join(r.output, "capsule-inputs.txt"), originals);
  const lock = hash(fs.readFileSync(path.join(r.workspace, "package-lock.json")));
  const image = hash(fs.readFileSync(r.native.path));
  const backend = hash(fs.readFileSync(r.backend));
  const results: unknown[] = [];
  try {
    const project = baseProject(r);
    for (const [name, stage] of [
      ["valid", "valid"],
      ["invalid", "invalid"],
      ["repaired", "valid"],
      ["proof-liveness", "poisoned"],
      ["proof-repaired", "valid"],
    ] as const) {
      const fixture = admissionFiles(stage);
      for (const [file, text] of Object.entries(fixture.files))
        fs.writeFileSync(path.join(project.directory, file), text);
      assert.equal(fixture.expected.length, stage === "valid" ? 0 : 30);
      write(path.join(r.output, "refs", name, "authored-inputs.txt"), fixture);
      results.push(
        ...checkProject(
          r,
          project.directory,
          name,
          fixture.expected,
          path.join(r.output, "refs", name),
        ),
      );
    }
    const valid = admissionFiles("valid"),
      invalid = admissionFiles("invalid");
    results.push(
      await lspAdmission(
        r,
        project,
        [
          { name: "valid", ...valid },
          { name: "invalid", ...invalid },
          { name: "repaired", ...valid },
        ],
        invalid.expected,
        true,
      ),
    );
    assert.equal(globalRefAdmission.length, 9);
    for (const [index, row] of globalRefAdmission.entries()) {
      const p = globalProject(r, index),
        mutation = mutatedAdmission(row.source, row.mutation);
      const invalidFiles = { ...p.files, "Probe.vue": mutation.source };
      for (const [name, files, expected] of [
        ["valid", p.files, []],
        ["invalid", invalidFiles, p.expected],
        ["repaired", p.files, []],
      ] as const) {
        for (const [file, text] of Object.entries(files))
          fs.writeFileSync(path.join(p.directory, file), text);
        write(path.join(r.output, "global-" + row.name, name, "authored-inputs.txt"), {
          files,
          expected,
        });
        results.push(
          ...checkProject(
            r,
            p.directory,
            name,
            [...expected],
            path.join(r.output, "global-" + row.name, name),
          ),
        );
      }
      results.push(
        await lspAdmission(
          r,
          p,
          [
            { name: "valid", files: p.files, expected: [] },
            { name: "invalid", files: invalidFiles, expected: p.expected },
            { name: "repaired", files: p.files, expected: [] },
          ],
          p.expected,
          false,
        ),
      );
    }
    assert.equal(results.length, 74);
    assert.equal(hash(fs.readFileSync(path.join(r.workspace, "package-lock.json"))), lock);
    assert.equal(hash(fs.readFileSync(r.native.path)), image);
    assert.equal(hash(fs.readFileSync(r.backend)), backend);
    write(path.join(r.output, "acceptance.txt"), {
      version: r.version,
      candidate: r.candidate,
      originals,
      harness: {
        source: process.env.GITHUB_SHA ?? null,
        run: process.env.GITHUB_RUN_ID ?? null,
        attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
        node: process.version,
      },
      sourceContract: { pullRequest: 10950, residualOriginIssue: 10980 },
      counts: {
        refCliStages: 5,
        refMapperStages: 5,
        uniqueMutations: 30,
        poisonedProofs: 30,
        globals: 9,
        globalCompilerStages: 54,
        lspDocuments: 39,
        lspAwaitedPackets: 117,
        typedHovers: 4,
      },
      results,
      passed: true,
    });
  } catch (error) {
    write(path.join(r.output, "failure.txt"), {
      error:
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : String(error),
      completedResults: results,
    });
    throw error;
  }
}
await main();
