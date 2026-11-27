import assert from "node:assert/strict";
import { h } from "vue";
import { Chat, ChatComposer, ChatMessage } from "../components/chat/index.ts";
import { ColorWheel } from "../components/color-wheel/index.ts";
import { Descriptions, DescriptionsItem } from "../components/descriptions/index.ts";
import { DiffView } from "../components/diff-view/index.ts";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupTextarea,
} from "../components/input-group/index.ts";
import { Result } from "../components/result/index.ts";
import { Segmented, SegmentedItem } from "../components/segmented/index.ts";
import { Statistic } from "../components/statistic/index.ts";
import { Watermark } from "../components/watermark/index.ts";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

function fixture(
  family: string,
  name: string,
  render: RuntimeFixture["render"],
  tag: string,
  attributes: Readonly<Record<string, string>>,
  text?: string,
): RuntimeFixture {
  return {
    name,
    sourceFile: `components/${family}/${name}.vue`,
    render,
    assertServerMarkup(html) {
      const opening = new RegExp(`<${tag}\\b[^>]*data-vize-ui="${name}"[^>]*>`).exec(html);
      assert.ok(opening, `${name} emits its native ${tag} root`);
      for (const [key, value] of Object.entries(attributes))
        assert.ok(opening[0].includes(`${key}="${value}"`), `${name} server ${key}`);
      if (text) assert.ok(html.includes(text), `${name} server content`);
    },
    assertHydratedDom(host) {
      const element = host.querySelector(`[data-vize-ui="${name}"]`);
      assert.ok(element instanceof HTMLElement);
      assert.equal(element.tagName, tag.toUpperCase());
      for (const [key, value] of Object.entries(attributes))
        assert.equal(element.getAttribute(key), value, `${name} hydrated ${key}`);
      if (text && element.tagName === "TEXTAREA")
        assert.equal((element as HTMLTextAreaElement).value, text, `${name} live value`);
      else if (text) assert.ok(element.textContent?.includes(text), `${name} hydrated content`);
    },
  };
}

const chat = () =>
  h(Chat, { label: "Support" }, () => [
    h(ChatMessage, { author: "Ada" }, () => "Build complete"),
    h(ChatComposer),
  ]);
const descriptions = () =>
  h(Descriptions, { label: "Build details" }, () =>
    h(DescriptionsItem, { label: "State" }, () => "Passed"),
  );
const group = () =>
  h(InputGroup, { label: "Website" }, () => [
    h(InputGroupAddon, null, () => "https://"),
    h(InputGroupInput, { modelValue: "vize.dev", "aria-label": "Address" }),
    h(InputGroupTextarea, { modelValue: "Release notes", "aria-label": "Notes" }),
    h(InputGroupButton, null, () => "Copy"),
  ]);
const segmented = () =>
  h(Segmented, { modelValue: "day", label: "Billing" }, () => [
    h(SegmentedItem, { value: "day" }, () => "Day"),
    h(SegmentedItem, { value: "week" }, () => "Week"),
  ]);

export const componentRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("chat", "chat", chat, "div", { role: "log", "aria-label": "Support" }, "Build complete"),
  fixture("chat", "chat-message", chat, "article", {}, "Build complete"),
  fixture("chat", "chat-composer", chat, "form", { "aria-label": "Composer" }, "Send"),
  fixture(
    "color-wheel",
    "color-wheel",
    () => h(ColorWheel, { modelValue: 180, label: "Hue" }),
    "div",
    { role: "slider", "aria-label": "Hue", "aria-valuenow": "180" },
  ),
  fixture(
    "descriptions",
    "descriptions",
    descriptions,
    "dl",
    { role: "group", "aria-label": "Build details" },
    "Passed",
  ),
  fixture(
    "descriptions",
    "descriptions-item",
    descriptions,
    "div",
    { role: "group", "aria-label": "State" },
    "Passed",
  ),
  fixture(
    "diff-view",
    "diff-view",
    () =>
      h(DiffView, {
        label: "Changes",
        patch: "--- a/readme.md\n+++ b/readme.md\n@@ -1 +1 @@\n-old\n+new\n",
      }),
    "table",
    { "aria-label": "Changes" },
    "new",
  ),
  fixture(
    "input-group",
    "input-group",
    group,
    "div",
    { role: "group", "aria-label": "Website" },
    "Copy",
  ),
  fixture("input-group", "input-group-addon", group, "span", {}, "https://"),
  fixture("input-group", "input-group-input", group, "input", {
    "aria-label": "Address",
    value: "vize.dev",
  }),
  fixture(
    "input-group",
    "input-group-textarea",
    group,
    "textarea",
    { "aria-label": "Notes" },
    "Release notes",
  ),
  fixture("input-group", "input-group-button", group, "button", { type: "button" }, "Copy"),
  fixture(
    "result",
    "result",
    () => h(Result, { title: "Published", status: "success" }),
    "div",
    { role: "status" },
    "Published",
  ),
  fixture(
    "segmented",
    "segmented",
    segmented,
    "div",
    { role: "radiogroup", "aria-label": "Billing" },
    "Day",
  ),
  fixture(
    "segmented",
    "segmented-item",
    segmented,
    "button",
    { role: "radio", "aria-checked": "true" },
    "Day",
  ),
  fixture(
    "statistic",
    "statistic",
    () => h(Statistic, { title: "Downloads", value: 12345, locale: "en-US" }),
    "div",
    { role: "group", "aria-label": "Downloads" },
    "12,345",
  ),
  fixture(
    "watermark",
    "watermark",
    () => h(Watermark, { text: "DRAFT" }, () => h("p", "Preview")),
    "div",
    {},
    "Preview",
  ),
];
