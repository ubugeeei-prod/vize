// Hosted execution consumes fresh Rust modules only, with independent HTML/tree authority.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { environment7502 } from "./native-attribute-values-7502-loader.ts";
import { hash7502, loadInputs7502 } from "./native-attribute-values-7502-inputs.ts";
import { validateCapture7502 } from "./native-attribute-values-7502-oracle.ts";
import { primary7502 } from "./native-attribute-values-7502-primary.ts";

const incorrectRc9 = [
  "reporter-7502",
  "original-regression-1",
  "original-regression-2",
  "original-regression-3",
  "original-regression-4",
  "original-regression-6",
  "original-regression-10",
  "original-regression-11",
  "original-regression-12",
];
export async function runtime7502(packet: any, mode: "development" | "production") {
  validateCapture7502(packet);
  const environment = await environment7502(mode);
  const { load, vaporRuntime, htmlTree, hostTree } = environment;
  const descendants = (host: any) => {
    const nodes: any[] = [];
    const walk = (parent: any) => {
      for (const node of parent.childNodes) {
        nodes.push(node);
        walk(node);
      }
    };
    walk(host);
    return nodes;
  };
  function consoleDiagnostics(diagnostics: string[]) {
    const previous = { warn: console.warn, error: console.error };
    console.warn = (...args) => diagnostics.push(args.map(String).join(" "));
    console.error = (...args) => diagnostics.push(args.map(String).join(" "));
    return () => {
      console.warn = previous.warn;
      console.error = previous.error;
    };
  }
  async function mount(loaded: any, target: string, html: string | null = null) {
    const host = document.body.appendChild(document.createElement("div"));
    if (html !== null) host.innerHTML = html;
    const originals = descendants(host);
    const initialTree = hostTree(host);
    const vue = loaded.runtime;
    const create =
      target === "vapor"
        ? html === null
          ? vue.createVaporApp
          : vue.createVaporSSRApp
        : html === null
          ? vue.createApp
          : vue.createSSRApp;
    const app = create(loaded.component, {});
    assert.equal(
      app._component.render,
      loaded.component.render,
      "real loaded default render identity",
    );
    const diagnostics: string[] = [];
    app.config.warnHandler = (message: string) => diagnostics.push(message);
    app.config.errorHandler = (error: any) => diagnostics.push(String(error));
    const restore = consoleDiagnostics(diagnostics);
    try {
      app.mount(host);
      await vue.nextTick();
      const nodes = descendants(host);
      const tree = hostTree(host);
      const retained = originals.map((node) => host.contains(node));
      if (html !== null)
        assert(retained.every(Boolean), "every real original SSR node survives hydration");
      app.unmount();
      await vue.nextTick();
      const unmounted = hostTree(host);
      assert.deepEqual(diagnostics, []);
      assert.deepEqual(unmounted, []);
      return {
        nodes,
        trace: {
          initialTree,
          tree,
          diagnostics,
          originalNodes: originals.length,
          retainedOriginalNodes: retained.filter(Boolean).length,
          retained,
          unmounted,
          renderIdentity: true,
        },
      };
    } finally {
      restore();
      host.remove();
    }
  }
  async function render(loaded: any) {
    const diagnostics: string[] = [];
    const app = loaded.runtime.createSSRApp(loaded.component, {});
    assert.equal(app._component.ssrRender, loaded.component.ssrRender);
    app.config.warnHandler = (message: string) => diagnostics.push(message);
    app.config.errorHandler = (error: any) => diagnostics.push(String(error));
    const restore = consoleDiagnostics(diagnostics);
    try {
      const html = await loaded.renderer.renderToString(app);
      assert.deepEqual(diagnostics, []);
      return { html, tree: htmlTree(html), diagnostics, renderIdentity: true };
    } finally {
      restore();
    }
  }
  const controls: any[] = [],
    executions: any[] = [];
  try {
    for (const fixture of loadInputs7502().fixtures) {
      const expected = htmlTree(fixture.source);
      assert.deepEqual(
        htmlTree(fixture.existingExpectedTemplateHtml),
        expected,
        "unchanged original legacy HTML semantic authority",
      );
      const factory = vaporRuntime.template(fixture.existingExpectedTemplateHtml, 3);
      const first = factory(),
        clone = factory();
      assert.notEqual(first, clone, "real template API creates distinct clones");
      assert.deepEqual([environment.tree(first)], expected);
      assert.deepEqual([environment.tree(clone)], expected);
      const primary = await primary7502(`<template>${fixture.source}</template>`);
      const primaryDom = await load(primary.dom.code, "dom", primary.dom.helperCode);
      const primarySsr = await load(primary.ssr.code, "ssr", primary.ssr.helperCode);
      const domMount = await mount(primaryDom, "dom");
      const ssr = await render(primarySsr);
      assert.deepEqual(domMount.trace.tree, expected);
      assert.deepEqual(ssr.tree, expected);
      const primaryVapor = await load(primary.vapor.code, "vapor", primary.vapor.helperCode);
      const primaryVaporServer = await load(
        primary.vapor.serverCode,
        "vapor-ssr",
        primary.vapor.helperCode,
      );
      const vaporSsr = await render(primaryVaporServer);
      assert.deepEqual(vaporSsr.tree, expected);
      const vaporMount = await mount(primaryVapor, "vapor");
      const vaporHydration = await mount(primaryVapor, "vapor", vaporSsr.html);
      assert.deepEqual(htmlTree(vaporSsr.html), expected);
      assert.deepEqual(vaporHydration.trace.tree, expected);
      const stockClientMatches = JSON.stringify(vaporMount.trace.tree) === JSON.stringify(expected);
      assert.equal(
        stockClientMatches,
        !incorrectRc9.includes(fixture.id),
        "retain precise known primary rc.9 negative controls",
      );
      controls.push({
        id: fixture.id,
        source: fixture.source,
        existingExpectedTemplateHtml: fixture.existingExpectedTemplateHtml,
        expectedTree: expected,
        directTemplate: {
          first: environment.tree(first),
          clone: environment.tree(clone),
          distinct: true,
        },
        primary,
        stockDomMount: domMount.trace,
        stockSsr: ssr,
        stockVaporSsr: vaporSsr,
        stockVaporMount: vaporMount.trace,
        stockVaporHydration: vaporHydration.trace,
        stockClientMatches,
        authority: "primary-only/no-native-credit",
      });
      for (const row of packet.rows.filter(
        (entry: any) => entry.id === fixture.id && entry.disposition === "positive",
      )) {
        const loaded = await load(row.result.code, row.target);
        if (row.target === "ssr") {
          const first = await render(loaded),
            repeat = await render(loaded);
          assert.deepEqual(first, repeat);
          assert.deepEqual(first.tree, expected);
          executions.push({
            id: row.id,
            target: row.target,
            linkMode: row.linkMode,
            codeSha256: hash7502(row.result.code),
            rendered: first,
            repeated: repeat,
          });
          continue;
        }
        const mounted = await mount(loaded, row.target),
          cloned = await mount(loaded, row.target);
        assert.deepEqual(mounted.trace.tree, expected);
        assert.deepEqual(cloned.trace.tree, expected);
        assert(
          cloned.nodes.length > 0 && cloned.nodes.every((node) => !mounted.nodes.includes(node)),
        );
        const hydrated = await mount(
          loaded,
          row.target,
          row.target === "vapor" ? vaporSsr.html : ssr.html,
        );
        assert.deepEqual(hydrated.trace.initialTree, expected);
        assert.deepEqual(hydrated.trace.tree, expected);
        executions.push({
          id: row.id,
          target: row.target,
          linkMode: row.linkMode,
          codeSha256: hash7502(row.result.code),
          mounted: mounted.trace,
          cloned: { ...cloned.trace, distinctNodes: true },
          hydrated: hydrated.trace,
        });
      }
    }
    assert.equal(controls.length, 14);
    assert.equal(executions.length, 72);
    return {
      schema: "vize.native-attribute-values-7502.runtime",
      version: 1,
      mode,
      capturedFromRust: true,
      acceptance: "unreviewed",
      nativeCodeSource: "fresh-source-built-capture",
      counts: {
        originalControls: 14,
        knownIncorrectRc9Clients: 9,
        positiveNativeExecutions: 72,
        lowerRefusalOutcomes: 12,
      },
      controls,
      executions,
    };
  } finally {
    await environment.window.happyDOM.close();
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 4);
  const mode = process.argv[3];
  assert(mode === "development" || mode === "production");
  const packet = JSON.parse(readFileSync(process.argv[2], "utf8"));
  const output = await runtime7502(packet, mode);
  process.stdout.write(`${JSON.stringify(output)}\n`);
}
