import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createCx, cx, defineVariants } from "./variants.ts";

const button = defineVariants({
  base: "btn",
  variants: {
    intent: { primary: "btn-primary", danger: "btn-danger" },
    size: { sm: "text-sm", md: "text-md" },
    disabled: { true: "opacity-50", false: "cursor-pointer" },
  },
  compoundVariants: [
    { intent: "danger", size: ["md"], class: "shadow" },
    { intent: "primary", disabled: true, class: "grayscale" },
  ],
  defaultVariants: { intent: "primary", size: "md" },
});

test("cx joins strings, numbers, nested lists, and truthy dictionary keys", () => {
  assert.equal(
    cx("a", 0, 1, 2n, null, undefined, false, true, "", ["b", ["c"]], { d: true, e: 0, f: "x" }),
    "a 0 1 2 b c d f",
  );
  assert.equal(cx(), "");
});

test("createCx applies the merge hook to the joined output", () => {
  const dedupe = createCx((value) => [...new Set(value.split(" "))].join(" "));
  assert.equal(dedupe("p-2", ["p-2", "m-1"]), "p-2 m-1");
});

test("recipes apply base, defaults, variants, compounds, then class in order", () => {
  assert.equal(button(), "btn btn-primary text-md cursor-pointer");
  assert.equal(
    button({ intent: "danger", class: ["extra", { on: true }] }),
    "btn btn-danger text-md cursor-pointer shadow extra on",
  );
  assert.equal(button({ disabled: true }), "btn btn-primary text-md opacity-50 grayscale");
  assert.deepEqual(button.variantKeys, ["intent", "size", "disabled"]);
});

test("unknown runtime options are ignored and undefined boolean variants use false", () => {
  const loose = button as (props: Record<string, unknown>) => string;
  assert.equal(loose({ intent: "ghost", size: undefined }), "btn text-md cursor-pointer");
});

test("merge hook post-processes recipe output", () => {
  const merged = defineVariants(
    { base: "p-2", variants: { dense: { true: "p-1" } } },
    { merge: (value) => value.replace("p-2 p-1", "p-1") },
  );
  assert.equal(merged({ dense: true }), "p-1");
});

test("slot recipes return per-slot class functions with overrides", () => {
  const card = defineVariants({
    base: "card",
    slots: { header: "card-header", body: "card-body" },
    variants: {
      tone: {
        plain: { header: "border-b" },
        loud: { base: "ring", body: "font-bold" },
      },
    },
    compoundVariants: [{ tone: "loud", class: { header: "uppercase" } }],
    defaultVariants: { tone: "plain" },
  });
  const plain = card({ class: "root-extra" });
  assert.equal(plain.base(), "card root-extra");
  assert.equal(plain.header(), "card-header border-b");
  assert.equal(plain.body({ class: "p-4" }), "card-body p-4");

  const loud = card({ tone: "loud" });
  assert.equal(loud.base(), "card ring");
  assert.equal(loud.header(), "card-header uppercase");
  assert.equal(loud.body({ tone: "plain" }), "card-body");
  assert.deepEqual(card.slotKeys, ["base", "header", "body"]);
});

test("responsive values prefix classes per breakpoint and reject unknown breakpoints", () => {
  const stack = defineVariants(
    {
      base: "flex",
      variants: { direction: { row: "flex-row gap-2", column: "flex-col" } },
      compoundVariants: [{ direction: "column", class: "items-start" }],
      responsive: ["sm", "md"],
    },
    { responsiveSeparator: ":" },
  );
  assert.equal(
    stack({ direction: { initial: "column", md: "row" } }),
    "flex flex-col md:flex-row md:gap-2 items-start",
  );
  const loose = stack as (props: Record<string, unknown>) => string;
  assert.throws(() => loose({ direction: { xl: "row" } }), /VIZE_UI_VARIANTS_BREAKPOINT/);
});

test("splitVariantProps separates variant props from forwarded props", () => {
  const [variants, rest] = button.splitVariantProps({ intent: "danger", id: "x", size: "sm" });
  assert.deepEqual(variants, { intent: "danger", size: "sm" });
  assert.deepEqual(rest, { id: "x" });
});

const Probe = defineComponent(() => () => h("button", { class: button({ intent: "danger" }) }));

test("renders identical classes on the server", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<button class="btn btn-danger text-md cursor-pointer shadow"></button>',
  );
});

test("hydrates server markup without mismatch diagnostics", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  try {
    const app = createSSRApp(Probe);
    app.mount(host);
    await nextTick();
    assert.equal(host.innerHTML, html);
    app.unmount();
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
