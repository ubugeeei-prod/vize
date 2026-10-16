import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  observeReaped,
  type ProcessIdentity,
} from "../performance/support/warm-type-backed-processes.ts";
import { hoverToText } from "./support/lsp/assertions.ts";
import { LspSession } from "./support/lsp/session.ts";
import { retiredOwnerWitness } from "./support/lsp/retired-owner-witness.ts";

const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

/** Observe actual direct children across all Vize worker threads, retaining both full stat reads. */
function directNativeChildren(parent: number, executable: string) {
  const candidates = new Set<number>();
  for (const task of fs.readdirSync(`/proc/${parent}/task`)) {
    try {
      for (const pid of fs
        .readFileSync(`/proc/${parent}/task/${task}/children`, "utf8")
        .trim()
        .split(/\s+/u))
        if (/^\d+$/u.test(pid)) candidates.add(Number(pid));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  const children = [];
  for (const pid of candidates) {
    try {
      const before = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
      const fields = before
        .slice(before.lastIndexOf(") ") + 2)
        .trim()
        .split(/\s+/u);
      if (Number(fields[1]) !== parent || fs.realpathSync(`/proc/${pid}/exe`) !== executable)
        continue;
      const after = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
      const later = after
        .slice(after.lastIndexOf(") ") + 2)
        .trim()
        .split(/\s+/u);
      assert.match(fields[19], /^\d+$/u);
      assert.equal(later[19], fields[19], "native birth changed during identity observation");
      assert.equal(Number(later[1]), parent);
      assert.notEqual(later[0], "Z", "the semantic provider must be live before termination");
      const command = fs.readFileSync(`/proc/${pid}/cmdline`);
      children.push({
        pid,
        birth_ticks: Number(fields[19]),
        parent,
        executable,
        executableSha256: digest(fs.readFileSync(`/proc/${pid}/exe`)),
        commandBase64: command.toString("base64"),
        commandArguments: command.toString("utf8").split("\0").filter(Boolean),
        wholeStatBefore: before,
        wholeStatAfter: after,
      });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  return children;
}

for (const mode of ["shutdown-exit", "exit", "eof"] as const) {
  test(
    `real stdio ${mode} reaps its same-birth native semantic owner`,
    {
      skip: process.platform !== "linux",
      timeout: 30000,
    },
    async () => {
      const manifest = createRequire(import.meta.url).resolve(
        "@typescript/typescript-linux-x64/package.json",
      );
      const metadata = JSON.parse(fs.readFileSync(manifest, "utf8"));
      assert.equal(metadata.version, "7.0.2");
      const native = fs.realpathSync(path.join(path.dirname(manifest), "lib/tsc"));
      const provider = process.env.VIZE_EDITOR_CONFIG_VUE_ROOT;
      assert.ok(provider, "reuse the native LSP action's installed Vue provider");
      const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-retirement-"));
      fs.mkdirSync(path.join(workspace, "node_modules"));
      fs.symlinkSync(fs.realpathSync(provider), path.join(workspace, "node_modules/vue"), "dir");
      fs.writeFileSync(
        path.join(workspace, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            target: "ESNext",
            module: "ESNext",
            moduleResolution: "Bundler",
            skipLibCheck: true,
          },
          include: ["*.vue"],
        }),
      );
      fs.writeFileSync(
        path.join(workspace, "vize.config.json"),
        JSON.stringify({
          lsp: { typecheck: true, lint: false },
          typeChecker: { corsaPath: native },
        }),
      );
      const source =
        '<script setup lang="ts">\nimport { ref } from "vue";\nconst value = ref(41);\nconst answer = value.value;\n</script>\n<template>{{ answer }}</template>\n';
      const file = path.join(workspace, "Owner.vue");
      fs.writeFileSync(file, source);
      const uri = pathToFileURL(file).href;
      const session = new LspSession();
      const observation: Record<string, unknown> = {
        mode,
        workspace,
        source,
        native,
        nativeSha256: digest(fs.readFileSync(native)),
        nativeManifest: metadata,
        vizePid: session.processId,
        fixtureFiles: Object.fromEntries(
          ["tsconfig.json", "vize.config.json", "Owner.vue"].map((name) => [
            name,
            fs.readFileSync(path.join(workspace, name), "utf8"),
          ]),
        ),
        vueProvider: {
          path: fs.realpathSync(provider),
          manifest: JSON.parse(fs.readFileSync(path.join(provider, "package.json"), "utf8")),
        },
      };
      let finished = false;
      try {
        observation.initialize = await session.initialize(workspace, {
          editor: true,
          typecheck: true,
          lint: false,
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        const query = { textDocument: { uri }, position: { line: 3, character: 8 } };
        const response = await session.request("textDocument/hover", query, 10000);
        observation.query = { method: "textDocument/hover", params: query, response };
        assert.match(hoverToText(response as Parameters<typeof hoverToText>[0]), /\bnumber\b/u);
        const children = directNativeChildren(session.processId, native);
        observation.nativeOwners = children;
        assert.equal(children.length, 2, "this fixture owns an API session and an editor LSP");
        assert.equal(
          children.filter((child) => child.commandArguments.includes("--api")).length,
          1,
          "the actual API owner must be present before termination",
        );
        assert.equal(
          children.filter((child) => child.commandArguments.includes("--lsp")).length,
          1,
          "the actual editor LSP owner must be present before termination",
        );
        for (const child of children)
          assert.equal(child.executableSha256, observation.nativeSha256);
        if (mode === "shutdown-exit") {
          observation.shutdownResponse = await session.request("shutdown", undefined, 5000);
          assert.equal(observation.shutdownResponse, null);
        }
        observation.serverExit = await session.finishInput(mode === "eof" ? "eof" : "exit");
        assert.deepEqual(observation.serverExit, { exitCode: 0, signal: null });
        const retired = await observeReaped(children as ProcessIdentity[]);
        observation.nativeReaping = retired;
        assert.equal(
          retired.reaped,
          true,
          `same-birth native life remains (including zombie): ${JSON.stringify(retired)}`,
        );
        finished = true;
      } catch (error) {
        observation.failure = error instanceof Error ? error.stack : String(error);
        throw error;
      } finally {
        observation.stderr = session.stderrText;
        const capture = process.env.VIZE_NATIVE_RETIREMENT_CAPTURE_DIR;
        if (capture) {
          fs.mkdirSync(capture, { recursive: true });
          fs.writeFileSync(
            path.join(capture, `${mode}.json`),
            `${JSON.stringify(observation, null, 2)}\n`,
          );
        }
        // Record the unmodified failure witness before any emergency test cleanup.
        if (!finished) await session.kill().catch(() => undefined);
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    },
  );
}

test(
  "retired in-flight native owner preserves typed custody across replacement and final shutdown",
  {
    skip: process.platform !== "linux",
    timeout: 30000,
  },
  async () => retiredOwnerWitness(directNativeChildren),
);

test(
  "killed old native owner retains typed failed-terminal custody before healthy replacement shutdown",
  { skip: process.platform !== "linux", timeout: 30000 },
  async () => retiredOwnerWitness(directNativeChildren, "killed"),
);
