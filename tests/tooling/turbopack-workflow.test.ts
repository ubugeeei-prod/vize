import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { parse } from "yaml";

void test(
  "the actual native unit capture propagates a failing producer through tee",
  {
    skip: process.platform === "win32" ? "the selected Actions job uses Linux bash" : false,
  },
  async () => {
    const workflow = parse(
      await fs.readFile(
        new URL("../../.github/workflows/turbopack-consumers.yml", import.meta.url),
        "utf8",
      ),
    );
    const step = workflow.jobs.consumers.steps.find(
      (step: { name?: string }) =>
        step.name === "Check authored positions and existing Rspack behavior",
    );
    assert.equal(typeof step?.run, "string");
    const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-unit-gate-"));
    try {
      const bin = path.join(directory, "bin");
      const artifacts = path.join(directory, "artifacts");
      await fs.mkdir(bin);
      const env = {
        ...process.env,
        PATH: `${bin}:${process.env.PATH}`,
        VIZE_TURBOPACK_ARTIFACTS: artifacts,
      };
      // The first command verifies this guard; the real native test producer is second.
      const script = path.join(bin, "node");
      await fs.writeFile(
        script,
        '#!/bin/sh\ncase "$*" in *turbopack-workflow.test.ts*) exit 0;; esac\nprintf "intentional producer failure\\n"\nexit 23\n',
        { mode: 0o755 },
      );
      const failed = spawnSync("/bin/bash", ["-e", "-c", step.run], { env, encoding: "utf8" });
      assert.equal(failed.status, 23, failed.stderr);
      assert.equal(
        await fs.readFile(path.join(artifacts, "source-unit.tap"), "utf8"),
        "intentional producer failure\n",
      );
      await fs.writeFile(script, '#!/bin/sh\nprintf "producer passed\\n"\nexit 0\n', {
        mode: 0o755,
      });
      const passed = spawnSync("/bin/bash", ["-e", "-c", step.run], { env, encoding: "utf8" });
      assert.equal(passed.status, 0, passed.stderr);
      assert.equal(
        await fs.readFile(path.join(artifacts, "source-unit.tap"), "utf8"),
        "producer passed\n",
      );
    } finally {
      await fs.rm(directory, { recursive: true, force: true });
    }
  },
);
