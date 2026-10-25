const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { isDeepStrictEqual } = require("node:util");
const vscode = require("vscode");
const {
  frames,
  records,
  publication,
  publicCompletion,
  retain,
} = require("./router-host-packets.cjs");

const source = `<script setup lang="ts">
// 雪😀
const stable = 1
</script>
<template><div v-if="stable">{{ stable }}</div></template>
<style>.x { display: flex; }</style>
`;
const label = (item) => (typeof item.label === "string" ? item.label : item.label.label);

async function prepareLocalResolution(manifest) {
  const evidence = { source, stages: [], calls: [], files: [] };
  const save = () => retain("local-resolution", evidence);
  const invoke = async (command, ...args) => {
    const observation = {
      command,
      args,
      beforeFrame: frames().length,
      startedAt: new Date().toISOString(),
    };
    evidence.calls.push(observation);
    const started = performance.now();
    try {
      const value = await vscode.commands.executeCommand(command, ...args);
      observation.result =
        command === "vscode.executeCompletionItemProvider"
          ? { isIncomplete: value.isIncomplete, items: value.items.map(publicCompletion) }
          : value;
      return value;
    } catch (error) {
      observation.error = { message: error.message, code: error.code, data: error.data };
      throw error;
    } finally {
      observation.afterFrame = frames().length;
      observation.elapsedMs = performance.now() - started;
      save();
    }
  };
  const request = (method, params) => invoke("vize.test.executeLspRequest", { method, params });
  const controls = [];
  const position = (document, needle, length) =>
    document.positionAt(document.getText().indexOf(needle) + length);
  for (const project of manifest.projects) {
    const file = path.join(project.root, "src/Local 雪😀.vue");
    fs.writeFileSync(file, source);
    evidence.files.push({ path: file, source: fs.readFileSync(file, "utf8") });
    save();
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
    await vscode.window.showTextDocument(document, { preview: false });
    const event = await publication(document);
    assert.deepEqual(event.packet.params, {
      diagnostics: [],
      uri: document.uri.toString(),
      version: document.version,
    });
    controls.push({ project, document });
  }
  const snapshot = async (control, phase) => {
    const { document } = control;
    const uri = document.uri.toString();
    const directivePosition = position(document, "v-if", 2);
    const cssPosition = position(document, "flex", 2);
    const value = { phase, project: control.project.name, uri, native: {}, public: {} };
    evidence.stages.push(value);
    save();
    const directive = await request("textDocument/completion", {
      textDocument: { uri },
      position: directivePosition,
    });
    value.native.directiveVector = directive;
    save();
    value.native.local = directive.find((item) => item.label === "v-if");
    assert.ok(value.native.local);
    assert.equal(value.native.local.data, undefined, "this item has no native or CSS owner");
    value.native.resolvedLocal = await request("completionItem/resolve", value.native.local);
    assert.deepEqual(value.native.resolvedLocal, value.native.local);
    const css = await request("textDocument/completion", {
      textDocument: { uri },
      position: cssPosition,
    });
    value.native.cssVector = css;
    save();
    value.native.css = css.find((item) => item.label === "flex");
    assert.ok(value.native.css);
    assert.deepEqual(value.native.css.data, {
      vizeCss: { kind: "value", name: "flex", property: "display" },
    });
    value.native.resolvedCss = await request("completionItem/resolve", value.native.css);
    const insertion = { ...value.native.resolvedCss };
    delete insertion.documentation;
    const originalInsertion = { ...value.native.css };
    delete originalInsertion.documentation;
    assert.deepEqual(insertion, originalInsertion);
    assert.notEqual(value.native.resolvedCss.documentation, undefined);
    const publicLocal = await invoke(
      "vscode.executeCompletionItemProvider",
      document.uri,
      directivePosition,
      undefined,
      20,
    );
    const localItem = publicLocal.items.find((item) => label(item) === "v-if");
    assert.ok(localItem);
    value.public.local = publicCompletion(localItem);
    const publicCss = await invoke(
      "vscode.executeCompletionItemProvider",
      document.uri,
      cssPosition,
      undefined,
      20,
    );
    const cssItem = publicCss.items.find((item) => label(item) === "flex");
    assert.ok(cssItem);
    value.public.css = publicCompletion(cssItem);
    assert.notEqual(value.public.css.documentation, null);
    save();
    return value;
  };
  for (const control of controls) control.baseline = await snapshot(control, "initial");

  const refuse = async (item) => {
    const start = frames().length;
    let failure;
    try {
      await request("completionItem/resolve", item);
    } catch (error) {
      failure = error;
    }
    const traffic = frames().slice(start);
    const sent = traffic.find(
      ({ direction, packet }) =>
        direction === "client" &&
        packet.method === "completionItem/resolve" &&
        isDeepStrictEqual(packet.params, item),
    );
    const response = traffic.find(
      ({ direction, packet }) => direction === "server" && packet.id === sent?.packet.id,
    );
    evidence.stages.push({ phase: "ownership-refusal", item, sent, response });
    save();
    assert.ok(sent && response);
    assert.deepEqual(response.packet, {
      jsonrpc: "2.0",
      id: sent.packet.id,
      error: { code: -32801, message: "Content modified" },
    });
    assert.ok(failure);
  };

  const afterWatch = async (on, after) => {
    // The caller has already retained the real OS tsconfig notification and
    // all original Router/Art captures; this helper never sends a watch event.
    const event = await publication(controls[0].document, after);
    evidence.stages.push({ phase: `config-${on ? "on" : "off"}-publication`, event });
    save();
    assert.deepEqual(event.packet.params, {
      diagnostics: [],
      uri: controls[0].document.uri.toString(),
      version: controls[0].document.version,
    });
    for (const control of controls) {
      const current = await snapshot(control, on ? "config-on" : "config-off");
      for (const key of ["local", "resolvedLocal", "css", "resolvedCss"])
        assert.deepEqual(current.native[key], control.baseline.native[key]);
      assert.deepEqual(current.public, control.baseline.public);
      assert.deepEqual(
        await request("completionItem/resolve", control.baseline.native.local),
        control.baseline.native.local,
      );
      assert.deepEqual(
        await request("completionItem/resolve", control.baseline.native.css),
        control.baseline.native.resolvedCss,
      );
    }
    const item = controls[0].baseline.native.css;
    for (const data of [
      { ...item.data, foreignOwner: { uri: controls[1].document.uri.toString() } },
      { vizeCss: { ...item.data.vizeCss, foreignOwner: null } },
      { vizeCss: null },
      { ...item.data, vizeCompletion: null },
    ])
      await refuse({ ...item, data });
    if (on) await nativeOwnerControl();
  };

  const nativeOwnerControl = async () => {
    const file = manifest.projects[0].pages.find((file) => !file.endsWith(".art.vue"));
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
    const original = document.getText();
    const params = {
      textDocument: { uri: document.uri.toString() },
      position: position(document, "route.params.page", "route.params.".length),
    };
    const completion = await request("textDocument/completion", params);
    const item = completion.find((item) => item.label === "page");
    assert.ok(item?.data?.vizeCompletion);
    assert.equal(vscode.Uri.parse(item.data.vizeCompletion.uri).fsPath, document.fileName);
    const resolved = await request("completionItem/resolve", item);
    const insertion = (value) => {
      const result = { ...value };
      delete result.detail;
      delete result.documentation;
      return result;
    };
    assert.deepEqual(
      insertion(resolved),
      insertion(item),
      "URI-bound native item keeps its insertion identity after primary retirement",
    );
    const replace = async (text) => {
      const start = frames().length;
      const edit = new vscode.WorkspaceEdit();
      edit.replace(
        document.uri,
        new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
        text,
      );
      assert.equal(await vscode.workspace.applyEdit(edit), true);
      assert.equal(document.getText(), text);
      const event = await publication(document, start);
      evidence.stages.push({ phase: "native-owner-edit", text, event });
      save();
      assert.deepEqual(event.packet.params, {
        diagnostics: [],
        uri: document.uri.toString(),
        version: document.version,
      });
    };
    try {
      await replace(
        original.replace('<script setup lang="ts">', '<script setup lang="ts">\n// stale 雪😀'),
      );
      const compiler = path.join(process.env.VIZE_TEST_ROUTER_OUTPUT, "compiler-protocol.jsonl");
      const start = records(compiler).length;
      assert.deepEqual(
        await request("completionItem/resolve", item),
        item,
        "stale native revision returns the entire original item",
      );
      const forwarded = records(compiler)
        .slice(start)
        .filter(
          ({ kind, direction, packet }) =>
            kind === "frame" &&
            direction === "client" &&
            packet.method === "completionItem/resolve",
        );
      evidence.stages.push({ phase: "stale-native-owner", item, forwarded });
      save();
      assert.deepEqual(forwarded, [], "stale completion does not reach the native compiler");
    } finally {
      await replace(original);
    }
  };
  return { afterWatch };
}

module.exports = { prepareLocalResolution };
