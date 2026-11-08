import assert from "node:assert/strict";
import { h } from "vue";
import { test } from "vitest";

import { firstChild, findNodes, mountComponent, mountFresco } from "../testing/index.js";
import type { FrescoNode } from "../renderer.js";
import Alert from "./FrescoAlert.vue";
import Avatar from "./FrescoAvatar.vue";
import Breadcrumb from "./FrescoBreadcrumb.vue";
import Card from "./FrescoCard.vue";
import Header from "./FrescoHeader.vue";
import Tooltip from "./FrescoTooltip.vue";

function nodeText(node: FrescoNode): string {
  const propText = node.props.text;
  const own =
    node.text ??
    (typeof propText === "string" || typeof propText === "number" ? String(propText) : "");
  return `${own}${node.children.map((child) => nodeText(child)).join("")}`;
}

function visibleText(node: FrescoNode): string[] {
  return findNodes(node, (child) => typeof child.text === "string" && child.text.length > 0).map(
    (child) => child.text ?? "",
  );
}

test("Alert renders its typed visual defaults and overrides", () => {
  const mounted = mountComponent(Alert, { message: "Saved", type: "success" });
  const root = firstChild(mounted);

  assert.equal(root.props.border, "rounded");
  assert.equal(root.props.fg, "green");
  assert.deepEqual(root.props.style, { flexDirection: "column", padding: 1 });
  assert.deepEqual(visibleText(root), ["✓ ", "Success", "Saved"]);
  mounted.unmount();
});

test("Avatar derives initials and renders an optional status", () => {
  const mounted = mountComponent(Avatar, {
    name: "Ada Lovelace",
    size: "lg",
    status: "online",
  });
  const root = firstChild(mounted);

  assert.equal(root.props.border, "rounded");
  assert.equal(root.props.bg, "blue");
  assert.deepEqual(root.props.style, { flexDirection: "row", paddingLeft: 1, paddingRight: 1 });
  assert.deepEqual(visibleText(root), ["AL", "●"]);
  assert.equal(root.children.find((node) => node.text === "●")?.props.fg, "green");
  mounted.unmount();
});

test("Breadcrumb preserves icon, active, and separator styling", () => {
  const mounted = mountComponent(Breadcrumb, {
    items: [
      { key: "home", label: "Home", icon: "⌂" },
      { key: "settings", label: "Settings" },
    ],
  });
  const root = firstChild(mounted);
  const textNodes = findNodes(
    root,
    (node) => typeof node.text === "string" && node.text.length > 0,
  );

  assert.deepEqual(
    textNodes.map((node) => node.text),
    ["⌂ ", "Home", " > ", "Settings"],
  );
  assert.deepEqual(
    textNodes.map((node) => ({
      fg: node.props.fg,
      bold: node.props.bold,
      underline: node.props.underline,
    })),
    [
      { fg: "gray", bold: undefined, underline: undefined },
      { fg: "gray", bold: false, underline: true },
      { fg: "gray", bold: undefined, underline: undefined },
      { fg: "white", bold: true, underline: false },
    ],
  );
  mounted.unmount();
});

test("Card keeps header, content, and named footer slot order", () => {
  const mounted = mountFresco(() =>
    h(
      Card,
      { title: "Summary", subtitle: "Today", border: "double" },
      {
        default: () => h("text", { text: "Body" }),
        footer: () => h("text", { text: "Ready" }),
      },
    ),
  );
  const root = firstChild(mounted);

  assert.equal(root.props.border, "double");
  assert.deepEqual(root.props.style, { flexDirection: "column", padding: 1 });
  assert.deepEqual(visibleText(root), ["Summary", " - Today"]);
  assert.match(nodeText(root), /Summary - TodayBodyReady/u);
  mounted.unmount();
});

test("Header supports slot overrides and its optional border", () => {
  const mounted = mountFresco(() =>
    h(
      Header,
      { title: "Dashboard", subtitle: "Live", left: "fallback", borderBottom: true },
      {
        left: () => h("text", { text: "Menu" }),
        right: () => h("text", { text: "Help" }),
      },
    ),
  );
  const root = firstChild(mounted);

  assert.deepEqual(root.props.style, { flexDirection: "column" });
  assert.match(nodeText(root), /^MenuDashboard - LiveHelp/u);
  assert.ok(nodeText(root).endsWith("─".repeat(80)));
  assert.doesNotMatch(nodeText(root), /fallback/u);
  mounted.unmount();
});

test("Tooltip orders its content around the bubble and supports hidden mode", () => {
  const right = mountComponent(Tooltip, { text: "Help", position: "right" }, () =>
    h("text", { text: "Target" }),
  );
  const rightRoot = firstChild(right);
  const boxes = rightRoot.children.filter((node) => node.type === "box");
  assert.deepEqual(rightRoot.props.style, { flexDirection: "row" });
  assert.equal(nodeText(boxes[0]), "Target");
  assert.equal(nodeText(boxes[1]), "Help");
  assert.equal(boxes[1]?.props.border, "rounded");
  right.unmount();

  const hidden = mountComponent(Tooltip, { text: "Help", visible: false }, () =>
    h("text", { text: "Target" }),
  );
  assert.equal(nodeText(firstChild(hidden)), "Target");
  assert.equal(firstChild(hidden).props.border, undefined);
  hidden.unmount();
});
