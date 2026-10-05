import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "../../tooling/support/lsp/assertions.ts";
import { QueryRecorder } from "./warm-type-backed-measure.ts";
import { nativeIdentities, retireNative } from "./warm-type-backed-processes.ts";
import {
  assertTypedPackets,
  hoverText,
  object,
  requests,
  waitForVersion,
  type Packet,
} from "./warm-type-backed-packets.ts";
import { generateWorkspace, type runtimeIdentity } from "./warm-type-backed-source.ts";

export async function controls(
  recorder: QueryRecorder,
  workspace: string,
  source: string,
  runtime: ReturnType<typeof runtimeIdentity>,
) {
  const session = recorder.session;
  const uri = pathToFileURL(path.join(workspace, "src/c/Comp0.vue")).href;
  const inputs: Array<Record<string, unknown>> = [];
  const step = async (name: string, work: () => Promise<void>) => {
    try {
      await work();
    } catch (failure) {
      recorder.failures.push(`${name}: ${String(failure)}`);
    }
  };
  const change = async (version: number, text: string) => {
    inputs.push({ name: "open-source", version, uri, text });
    const start = session.stderrText.length;
    session.notify("textDocument/didChange", {
      textDocument: { uri, version },
      contentChanges: [{ text }],
    });
    await waitForVersion(session, uri, version, start);
  };
  const sweep = async (stage: string, text = source) => {
    const packets = await recorder.sweep(stage, requests(uri, text));
    assertTypedPackets(packets, workspace);
    return packets;
  };
  const edit = source.replace("const open = ref(false);", "const open = ref(0);");
  assert.notEqual(edit, source);
  await step("unsaved-host", async () => {
    await change(2, edit);
    assert.match(hoverText((await sweep("unsaved-host", edit))[0].result), /\bnumber\b/u);
  });
  await step("restored-host", async () => {
    await change(3, source);
    assert.match(hoverText((await sweep("restored-host"))[0].result), /\bboolean\b/u);
  });

  const rewrite = (file: string, text: string) => {
    const stamp = fs.statSync(file, { bigint: true });
    fs.writeFileSync(file, text);
    fs.utimesSync(file, 1_700_000_000, 1_700_000_000);
    assert.equal(fs.statSync(file, { bigint: true }).mtimeNs, stamp.mtimeNs);
    assert.equal(fs.statSync(file, { bigint: true }).size, stamp.size);
    inputs.push({
      name: "closed-disk",
      file,
      text,
      mtimeNs: String(stamp.mtimeNs),
      size: String(stamp.size),
    });
  };
  const resolveCount = async (stage: string, packets: Packet[], type: string) => {
    const completion = packets[3].result;
    const items = Array.isArray(completion) ? completion : object(completion).items;
    assert.ok(Array.isArray(items));
    const item = items.find((entry: unknown) => object(entry).label === "count");
    assert.ok(item, "the typed count completion is required");
    const resolved = await recorder.query(stage, {
      name: "resolved count",
      method: "completionItem/resolve",
      params: item,
    });
    const detail = object(resolved.result).detail;
    assert.equal(typeof detail, "string");
    assert.match(detail as string, new RegExp(`\\b${type}\\b`, "u"));
  };
  const dependency = path.join(workspace, "src/lib/mod0.ts");
  const originalDependency = fs.readFileSync(dependency, "utf8");
  const changedDependency = originalDependency.replace("count: Ref<number>", "count: Ref<string>");
  assert.notEqual(changedDependency, originalDependency);
  assert.equal(changedDependency.length, originalDependency.length);
  await step("same-stamp-dependency", async () => {
    rewrite(dependency, changedDependency);
    await resolveCount(
      "same-stamp-dependency-resolve",
      await sweep("same-stamp-dependency"),
      "string",
    );
  });
  await step("restored-dependency", async () => {
    rewrite(dependency, originalDependency);
    await resolveCount("restored-dependency-resolve", await sweep("restored-dependency"), "number");
  });

  const probe = source.replace(
    "const open = ref(false);",
    "const open = ref(false);\nconst configProbe = null;",
  );
  const probeSpec = {
    name: "configuration probe",
    method: "textDocument/hover",
    params: {
      textDocument: { uri },
      position: offsetToPosition(probe, probe.indexOf("configProbe") + 3),
    },
  };
  const config = path.join(workspace, "tsconfig.json");
  const originalConfig = fs.readFileSync(config, "utf8");
  const relaxed = originalConfig.replace('"strict":true', '"strict":false').trimEnd();
  assert.notEqual(relaxed, originalConfig);
  assert.equal(Buffer.byteLength(relaxed), Buffer.byteLength(originalConfig));
  let strict: Packet | undefined;
  await step("strict-config", async () => {
    await change(4, probe);
    strict = await recorder.query("strict-config", probeSpec);
    assert.match(hoverText(strict.result), /\bnull\b/u);
  });
  await step("same-stamp-config", async () => {
    rewrite(config, relaxed);
    assert.match(
      hoverText((await recorder.query("same-stamp-config", probeSpec)).result),
      /\bany\b/u,
    );
  });
  await step("restored-config", async () => {
    rewrite(config, originalConfig);
    const restored = await recorder.query("restored-config", probeSpec);
    assert.ok(strict);
    assert.deepEqual(restored.result, strict.result);
  });
  await step("after-config-restore", async () => {
    await change(5, source);
    await sweep("after-config-restore");
  });

  await step("cancel", async () => {
    await recorder.query("cancel", requests(uri, source)[0], (id) => [
      { jsonrpc: "2.0", method: "$/cancelRequest", params: { id } },
    ]);
  });
  await step("after-cancel", async () => {
    await sweep("after-cancel");
  });
  await step("closed-refusal", async () => {
    const closed = new Promise<void>((resolve) => {
      const observer = (method: string, params: unknown) => {
        if (method !== "textDocument/publishDiagnostics") return;
        const value = object(params);
        if (
          value.uri === uri &&
          value.version == null &&
          Array.isArray(value.diagnostics) &&
          value.diagnostics.length === 0
        ) {
          session.notificationObservers.splice(session.notificationObservers.indexOf(observer), 1);
          resolve();
        }
      };
      session.notificationObservers.push(observer);
    });
    session.notify("textDocument/didClose", { textDocument: { uri } });
    await Promise.race([
      closed,
      new Promise<void>((_, reject) => {
        const timeout = setTimeout(
          () => reject(new Error("closed-document diagnostics acknowledgement missing")),
          60_000,
        );
        void closed.then(() => clearTimeout(timeout));
      }),
    ]);
    assert.deepEqual(
      (await recorder.sweep("closed-refusal", requests(uri, source))).map(
        (packet) => packet.result,
      ),
      [null, null, null, null],
    );
  });
  await step("reopened-host", async () => {
    const start = session.stderrText.length;
    inputs.push({ name: "reopen-source", version: 1, uri, text: source });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await waitForVersion(session, uri, 1, start);
    await sweep("reopened-host");
  });

  const other = generateWorkspace(`${workspace}-other`, runtime, 20);
  inputs.push({ name: "cross-root-original20", ...other });
  await step("cross-root", async () => {
    session.notify("workspace/didChangeWorkspaceFolders", {
      event: {
        added: [{ uri: pathToFileURL(other.workspace).href, name: "original20" }],
        removed: [],
      },
    });
    const otherUri = pathToFileURL(path.join(other.workspace, "src/c/Comp0.vue")).href;
    const otherSource = other.source["src/c/Comp0.vue"];
    const start = session.stderrText.length;
    session.notify("textDocument/didOpen", {
      textDocument: { uri: otherUri, languageId: "vue", version: 1, text: otherSource },
    });
    await waitForVersion(session, otherUri, 1, start);
    assertTypedPackets(
      await recorder.sweep("cross-root", requests(otherUri, otherSource)),
      other.workspace,
    );
  });
  await step("original-root-after-foreign", async () => {
    await sweep("original-root-after-foreign");
  });
  await step("native-epoch-recovery", async () => {
    const previous = nativeIdentities(recorder.sampler.sample(), runtime.executable);
    const retired = await retireNative(
      () => recorder.sampler.sample(),
      runtime.executable,
      previous,
    );
    inputs.push({ name: "retired-native-epoch", processes: previous, ...retired });
    await change(2, source);
    await sweep("native-epoch-recovered");
    const current = nativeIdentities(recorder.sampler.sample(), runtime.executable);
    assert.ok(current.length > 0);
    assert.ok(
      current.every(
        (entry: { pid: number; birth_ticks: number }) =>
          !previous.some(
            (old: { pid: number; birth_ticks: number }) =>
              old.pid === entry.pid && old.birth_ticks === entry.birth_ticks,
          ),
      ),
    );
    inputs.push({ name: "recovered-native-epoch", processes: current });
  });
  return inputs;
}
