import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { InteractionHandle } from "../../../testing/mount.ts";
import { createRichTextCommands, isMarkActive } from "./rich-text-commands.ts";
import { richTextToHtml } from "./rich-text-html.ts";
import { domFromPosition, positionFromDom, writeDomSelection } from "./rich-text-dom.ts";
import type { RichTextDoc, RichTextSelection } from "./rich-text-model.ts";
import RichTextBubbleMenu from "./rich-text-bubble-menu.vue";
import RichTextContent from "./rich-text-content.vue";
import RichTextRoot from "./rich-text-root.vue";
import RichTextToolbar from "./rich-text-toolbar.vue";
import RichTextToolbarButton from "./rich-text-toolbar-button.vue";
import type { RichTextDefaultSchema } from "./rich-text-schema.ts";
import type { RtState } from "./rich-text-state.ts";
import { at, doc, h as heading, p, schema, t, ul } from "./rich-text-test-utils.ts";

type Doc = RichTextDoc<RichTextDefaultSchema>;
const commands = createRichTextCommands(schema);
const handles: InteractionHandle[] = [];

afterEach(() => {
  for (const handle of handles.splice(0)) handle.unmount();
});

const Editor = RichTextRoot<RichTextDefaultSchema>;

const Harness = defineComponent({
  props: {
    initial: { type: Object as PropType<Doc>, required: true },
    editable: { type: Boolean, default: true },
    controlled: { type: Boolean, default: false },
    log: { type: Array as PropType<string[]>, default: () => [] },
  },
  setup(props) {
    const value = ref<Doc>(props.initial);
    return () =>
      h("div", [
        h(
          Editor,
          {
            id: "editor",
            schema,
            editable: props.editable,
            ...(props.controlled ? { modelValue: value.value } : { defaultValue: props.initial }),
            "onUpdate:modelValue": (next: Doc) => {
              props.log.push(richTextToHtml(schema, next));
              if (props.controlled) value.value = next;
            },
          },
          () => [
            h(RichTextToolbar, { ariaLabel: "Format" }, () => [
              h(
                RichTextToolbarButton,
                {
                  command: commands.toggleMark("bold"),
                  active: (state: RtState) => isMarkActive(state, "bold"),
                  ariaLabel: "Bold",
                },
                () => "B",
              ),
              h(
                RichTextToolbarButton,
                { command: commands.toggleWrap("bulletList"), ariaLabel: "List" },
                () => "•",
              ),
              h(RichTextToolbarButton, { command: commands.undo, ariaLabel: "Undo" }, () => "↶"),
            ]),
            h(RichTextContent, { ariaLabel: "Body", placeholder: "Write…" }),
            h(RichTextBubbleMenu, null, () =>
              h(
                RichTextToolbarButton,
                { command: commands.toggleMark("italic"), ariaLabel: "Italic" },
                () => "I",
              ),
            ),
          ],
        ),
      ]);
  },
});

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index++) await nextTick();
}

function mount(
  initial: Doc,
  props: Record<string, unknown> = {},
): { handle: InteractionHandle; content: HTMLElement; log: string[] } {
  const log: string[] = [];
  const handle = mountInteraction(Harness, { props: { initial, log, ...props } });
  handles.push(handle);
  const content = document.querySelector('[data-vize-ui="rich-text-content"]');
  assert.ok(content instanceof HTMLElement);
  return { handle, content, log };
}

async function select(content: HTMLElement, selection: RichTextSelection): Promise<void> {
  content.focus();
  writeDomSelection(content, selection);
  document.dispatchEvent(new Event("selectionchange"));
  await settle();
}

async function input(
  content: HTMLElement,
  inputType: string,
  data: string | null = null,
): Promise<InputEvent> {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  content.dispatchEvent(event);
  await settle();
  return event;
}

function composition(type: string, data: string): CompositionEvent {
  const event = new CompositionEvent(type, { bubbles: true, data });
  // happy-dom drops CompositionEventInit.data; mirror the browser field.
  Object.defineProperty(event, "data", { value: data });
  return event;
}

async function key(target: Element, init: KeyboardEventInit): Promise<KeyboardEvent> {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  await settle();
  return event;
}

function button(label: string): HTMLButtonElement {
  const element = document.querySelector(`[aria-label="${label}"]`);
  assert.ok(element instanceof HTMLButtonElement, `expected ${label}`);
  return element;
}

function body(content: HTMLElement): string {
  return content.innerHTML
    .replaceAll(/ data-rt-[a-z]+="[^"]*"/gu, "")
    .replaceAll(' contenteditable="false"', "");
}

test("renders an accessible multiline text box with editor hooks", async () => {
  const { content } = mount(doc(p("")));
  await settle();
  assert.equal(content.getAttribute("role"), "textbox");
  assert.equal(content.getAttribute("aria-multiline"), "true");
  assert.equal(content.getAttribute("aria-label"), "Body");
  assert.equal(content.getAttribute("contenteditable"), "true");
  assert.equal(content.getAttribute("data-empty"), "true");
  assert.equal(content.getAttribute("aria-placeholder"), "Write…");
  assert.equal(content.id, "editor-content");
  assert.equal(
    document.querySelector('[role="toolbar"]')?.getAttribute("aria-controls"),
    "editor-content",
  );
  assert.match(
    content.innerHTML,
    /<p data-rt-path="0" data-rt-textblock=""><br data-rt-filler=""><\/p>/u,
  );
});

test("DOM and model positions round-trip through text, marks, and leaves", () => {
  const host = document.createElement("div");
  host.innerHTML = richTextToHtml(
    schema,
    doc(
      p(
        "ab",
        t("cd", "bold"),
        { type: "image", attrs: { src: "/x.png", alt: "" }, marks: [] },
        "e",
      ),
      ul([p("x")]),
    ),
    { editor: true },
  );
  document.body.append(host);
  for (const offset of [0, 1, 2, 3, 4, 5, 6]) {
    const point = domFromPosition(host, at([0], offset));
    assert.ok(point, `point ${offset}`);
    assert.deepEqual(positionFromDom(host, point.node, point.offset), at([0], offset));
  }
  const nested = domFromPosition(host, at([1, 0, 0], 1));
  assert.ok(nested);
  assert.deepEqual(positionFromDom(host, nested.node, nested.offset), at([1, 0, 0], 1));
  assert.deepEqual(
    positionFromDom(host, host, 1),
    at([1, 0, 0], 0),
    "element points snap to textblocks",
  );
  host.remove();
});

test("beforeinput typing goes through the model, re-renders, and emits v-model", async () => {
  const { content, log } = mount(doc(p("ac")));
  await settle();
  await select(content, { anchor: at([0], 1), head: at([0], 1) });
  const event = await input(content, "insertText", "b");
  assert.equal(event.defaultPrevented, true, "the browser never mutates the DOM itself");
  assert.equal(body(content), "<p>abc</p>");
  assert.deepEqual(log, ["<p>abc</p>"]);
  const selection = document.getSelection();
  assert.ok(selection?.anchorNode);
  assert.deepEqual(
    positionFromDom(content, selection.anchorNode, selection.anchorOffset),
    at([0], 2),
  );
});

test("input rules, Enter, Backspace, and line breaks follow the keymap", async () => {
  const { content } = mount(doc(p("")));
  await settle();
  await select(content, { anchor: at([0], 0), head: at([0], 0) });
  for (const character of "# Title") await input(content, "insertText", character);
  assert.equal(body(content), "<h1>Title</h1>");
  await key(content, { key: "Enter" });
  await input(content, "insertText", "x");
  assert.equal(body(content), "<h1>Title</h1><p>x</p>");
  await key(content, { key: "Backspace" });
  await key(content, { key: "Backspace" });
  assert.equal(body(content), "<h1>Title</h1>");
  await input(content, "insertLineBreak");
  assert.equal(body(content), "<h1>Title<br><br></h1>");
});

test("shortcuts format the selection and undo restores it", async () => {
  const { content } = mount(doc(p("bold me")));
  await settle();
  await select(content, { anchor: at([0], 0), head: at([0], 4) });
  const bold = await key(content, { key: "b", ctrlKey: true });
  assert.equal(bold.defaultPrevented, true);
  assert.equal(body(content), "<p><strong>bold</strong> me</p>");
  assert.equal(button("Bold").getAttribute("aria-pressed"), "true");
  await key(content, { key: "z", metaKey: true });
  assert.equal(body(content), "<p>bold me</p>");
  await input(content, "historyRedo");
  assert.equal(body(content), "<p><strong>bold</strong> me</p>");
});

test("paste parses sanitized HTML and plain text into the document", async () => {
  const { content } = mount(doc(p("ab")));
  await settle();
  await select(content, { anchor: at([0], 1), head: at([0], 1) });
  const paste = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(paste, "clipboardData", {
    value: {
      getData: (type: string) =>
        type === "text/html"
          ? "<p>X<script>bad()</script><em>y</em></p><ul><li>z</li></ul><img src=x onerror=alert(1)>"
          : "",
    },
  });
  content.dispatchEvent(paste);
  await settle();
  assert.equal(paste.defaultPrevented, true);
  assert.equal(
    body(content),
    '<p>aX<em>y</em></p><ul><li><p>z</p></li></ul><p><img src="x" alt="">b</p>',
  );
  assert.doesNotMatch(content.innerHTML, /script|onerror/u);
});

test("composition text is committed once at compositionend and the DOM is repaired", async () => {
  const { content, log } = mount(doc(p("a")));
  await settle();
  await select(content, { anchor: at([0], 1), head: at([0], 1) });
  content.dispatchEvent(composition("compositionstart", ""));
  const during = await input(content, "insertCompositionText", "か");
  assert.equal(during.defaultPrevented, false, "the IME owns the DOM while composing");
  const text = content.querySelector("p")?.firstChild;
  if (text) text.textContent = "aか";
  content.dispatchEvent(composition("compositionend", "漢"));
  await settle();
  assert.equal(body(content), "<p>a漢</p>");
  assert.deepEqual(log, ["<p>a漢</p>"]);
  content.dispatchEvent(composition("compositionstart", ""));
  if (content.querySelector("p")?.firstChild)
    content.querySelector("p")!.firstChild!.textContent = "garbage";
  content.dispatchEvent(composition("compositionend", ""));
  await settle();
  assert.equal(body(content), "<p>a漢</p>", "canceled compositions restore the model's DOM");
});

test("toolbar buttons reflect command state, keep focus, and rove with arrows", async () => {
  const { content } = mount(doc(p("text")));
  await settle();
  assert.equal(button("Undo").getAttribute("aria-disabled"), "true");
  assert.equal(button("Bold").getAttribute("tabindex"), "0");
  assert.equal(button("List").getAttribute("tabindex"), "-1");
  await select(content, { anchor: at([0], 0), head: at([0], 4) });
  const down = new MouseEvent("mousedown", { bubbles: true, cancelable: true });
  button("List").dispatchEvent(down);
  assert.equal(down.defaultPrevented, true, "pressing a tool keeps the text selection");
  button("List").click();
  await settle();
  assert.equal(body(content), "<ul><li><p>text</p></li></ul>");
  assert.equal(button("Undo").getAttribute("aria-disabled"), null);
  await key(content, { key: "F10", altKey: true });
  assert.ok(document.activeElement === button("Bold"), "Alt+F10 moves focus to the toolbar");
  await key(button("Bold"), { key: "ArrowRight" });
  assert.ok(document.activeElement === button("List"));
  await key(button("List"), { key: "End" });
  assert.ok(document.activeElement === button("Undo"));
  await key(button("Undo"), { key: "Escape" });
  assert.ok(document.activeElement === content, "Escape returns to the text");
});

test("the bubble menu appears for a focused non-empty selection", async () => {
  const { content } = mount(doc(p("pick")));
  await settle();
  const host = document.querySelector('[data-vize-ui="rich-text-bubble-menu-host"]');
  assert.ok(host instanceof HTMLElement);
  assert.equal(host.hidden, true);
  await select(content, { anchor: at([0], 0), head: at([0], 2) });
  assert.equal(host.hidden, false);
  const menu = document.querySelector('[data-vize-ui="rich-text-bubble-menu"]');
  assert.equal(menu?.getAttribute("role"), "toolbar");
  button("Italic").click();
  await settle();
  assert.equal(body(content), "<p><em>pi</em>ck</p>");
  await select(content, { anchor: at([0], 1), head: at([0], 1) });
  assert.equal(host.hidden, true);
});

test("read-only editors refuse input and disable tools", async () => {
  const { content, log } = mount(doc(p("fixed")), { editable: false });
  await settle();
  assert.equal(content.getAttribute("contenteditable"), "false");
  assert.equal(content.getAttribute("aria-readonly"), "true");
  await select(content, { anchor: at([0], 0), head: at([0], 5) });
  const event = await input(content, "insertText", "x");
  assert.equal(event.defaultPrevented, true);
  await key(content, { key: "b", ctrlKey: true });
  assert.equal(body(content), "<p>fixed</p>");
  assert.deepEqual(log, []);
  assert.equal(button("Bold").getAttribute("aria-disabled"), "true");
});

test("controlled documents follow v-model and external replacements", async () => {
  const { handle, content } = mount(doc(p("one")), { controlled: true });
  await settle();
  await select(content, { anchor: at([0], 3), head: at([0], 3) });
  await input(content, "insertText", "!");
  assert.equal(body(content), "<p>one!</p>");
  await handle.wrapper.setProps({ initial: doc(heading(2, "reset")) });
  await settle();
  assert.equal(body(content), "<p>one!</p>", "initial only seeds the controlled value");
});
