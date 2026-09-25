import assert from "node:assert/strict";

import { test, vi } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, useTemplateRef } from "vue";
import { renderToString } from "vue/server-renderer";

import type { A11yAuditIssue } from "./a11y-audit.ts";
import { accessibleNameOf, auditAccessibility } from "./a11y-audit-rules.ts";
import { useA11yAudit } from "./a11y-audit-runtime.ts";
import FloatingActionButton from "../../actions/floating-action-button/floating-action-button.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function fragment(html: string): HTMLDivElement {
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  return host;
}

test("reports unnamed interactive ui parts and accepts every naming technique", () => {
  const host = fragment(`
    <div data-vize-ui="toolbar">
      <button id="icon"><svg></svg></button>
      <button aria-label="Bold">B</button>
      <button aria-labelledby="bold-label"></button><span id="bold-label">Bold</span>
      <label for="size">Size</label><input id="size" />
      <label>Search <input /></label>
      <button title="Italic"></button>
      <span role="switch"></span>
      <input type="hidden" />
      <button hidden></button>
    </div>
    <button>outside ui parts</button><button></button>
  `);

  const issues = auditAccessibility(host);
  assert.deepEqual(
    issues.map((item) => [item.rule, item.element.tagName, item.part]),
    [
      ["accessible-name", "BUTTON", "toolbar"],
      ["accessible-name", "SPAN", "toolbar"],
    ],
  );
  assert.match(issues[0]?.message ?? "", /no accessible name/);
  assert.equal(auditAccessibility(host, { onlyUiParts: false }).length, 3);
  assert.equal(auditAccessibility(host, { ignore: ["accessible-name"] }).length, 0);

  host.remove();
});

test("flags unnamed dialogs, repeated or region landmarks, missing alt, and dangling idrefs", () => {
  const host = fragment(`
    <div data-vize-ui="shell">
      <div role="dialog" aria-describedby="nowhere"></div>
      <nav></nav><nav aria-label="Footer"></nav>
      <div role="region"></div>
      <img src="x.png" />
      <img src="y.png" alt="" />
    </div>
  `);

  const rules = auditAccessibility(host).map((item) => item.rule);
  assert.deepEqual(rules, [
    "dialog-name",
    "dangling-idref",
    "landmark-name",
    "landmark-name",
    "image-alt",
  ]);

  host.remove();
});

test("accessibleNameOf follows labelledby, label, alt, title, and content precedence", () => {
  const host = fragment(`
    <span id="lbl">From labelledby</span>
    <button id="a" aria-labelledby="lbl" aria-label="ignored">text</button>
    <button id="b"><img alt="Save" /></button>
    <input id="c" placeholder="Filter" />
  `);
  const get = (id: string) => host.querySelector(`#${id}`) ?? assert.fail(id);

  assert.equal(accessibleNameOf(get("a")), "From labelledby");
  assert.equal(accessibleNameOf(get("b")), "Save");
  assert.equal(accessibleNameOf(get("c")), "Filter");
  assert.throws(() => auditAccessibility(null as never), /VIZE_UI_A11Y_AUDIT/);

  host.remove();
});

test("useA11yAudit warns once per element and rule in development and re-audits on change", async () => {
  const warnings: unknown[][] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => warnings.push(values);
  const Probe = defineComponent({
    name: "A11yAuditProbe",
    setup() {
      const root = useTemplateRef<HTMLDivElement>("root");
      const audit = useA11yAudit(root);
      return () =>
        h("div", { ref: "root", "data-issues": String(audit.issues.value.length) }, [
          h(FloatingActionButton, null, () => h("svg")),
        ]);
    },
  });
  try {
    const handle = mountInteraction(Probe);
    await nextTick();
    assert.equal(warnings.length, 1);
    assert.match(
      String(warnings[0]?.[0]),
      /^\[VIZE_UI_A11Y_AUDIT\] accessible-name: <button> in floating-action-button/,
    );

    const extra = document.createElement("button");
    extra.setAttribute("data-vize-ui", "extra");
    handle.root().append(extra);
    await Promise.resolve();
    await nextTick();
    assert.equal(warnings.length, 2, "a new unnamed part is reported after the mutation");
    handle.root().setAttribute("data-touch", "1");
    await Promise.resolve();
    assert.equal(warnings.length, 2, "known issues are not reported twice");
    handle.unmount();
  } finally {
    console.warn = originalWarn;
  }
});

test("custom onIssue handlers receive issues and audit() runs on demand", async () => {
  const seen: A11yAuditIssue[] = [];
  let run: (() => readonly A11yAuditIssue[]) | null = null;
  const Probe = defineComponent({
    name: "A11yAuditHandlerProbe",
    setup() {
      const root = useTemplateRef<HTMLDivElement>("root");
      const audit = useA11yAudit(root, { observe: false, onIssue: (item) => seen.push(item) });
      run = audit.audit;
      return () => h("div", { ref: "root" }, h("button", { "data-vize-ui": "button" }));
    },
  });
  const handle = mountInteraction(Probe);
  await nextTick();

  assert.equal(seen.length, 1);
  assert.equal(seen[0]?.rule, "accessible-name");
  assert.equal(run?.().length, 1);
  assert.equal(seen.length, 1);
  handle.unmount();
  assert.throws(() => useA11yAudit(null), /VIZE_UI_A11Y_AUDIT/);
});

test("server rendering never audits", async () => {
  const warnings: unknown[][] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => warnings.push(values);
  try {
    const Probe = defineComponent({
      name: "A11yAuditSsrProbe",
      setup() {
        const root = useTemplateRef<HTMLDivElement>("root");
        useA11yAudit(root);
        return () => h("div", { ref: "root" }, h("button", { "data-vize-ui": "button" }));
      },
    });
    const html = await renderToString(createSSRApp(Probe));
    assert.equal(html, '<div><button data-vize-ui="button"></button></div>');
    assert.deepEqual(warnings, []);
  } finally {
    console.warn = originalWarn;
  }
});

test("production builds disable the auditor", async () => {
  const original = process.env.NODE_ENV;
  process.env.NODE_ENV = "production";
  vi.resetModules();
  try {
    const runtime = await import("./a11y-audit-runtime.ts");
    assert.equal(runtime.a11yAuditEnabled, false);
    let enabled = true;
    const Probe = defineComponent({
      name: "A11yAuditProductionProbe",
      setup() {
        const audit = runtime.useA11yAudit(() => null);
        enabled = audit.enabled;
        return () => h("div", h("button"));
      },
    });
    const handle = mountInteraction(Probe);
    assert.equal(enabled, false);
    handle.unmount();
  } finally {
    process.env.NODE_ENV = original;
    vi.resetModules();
  }
});
