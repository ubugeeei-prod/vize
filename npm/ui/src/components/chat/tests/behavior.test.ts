import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import ChatComposer from "../chat-composer.vue";
import ChatMessage from "../chat-message.vue";
import Chat from "../chat.vue";
import { mountInteraction } from "../../../testing/mount.ts";

async function typeMessage(box: HTMLTextAreaElement, value: string): Promise<void> {
  box.value = value;
  box.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
}

test("renders a log of authored messages", () => {
  const handle = mountInteraction(Chat, {
    props: { label: "Support" },
    slots: {
      default: () => [
        h(ChatMessage, { author: "Ada" }, () => "The build is green."),
        h(ChatComposer),
      ],
    },
  });
  const log = handle.getByRole("log", { name: "Support" });

  assert.equal(log.getAttribute("data-vize-ui"), "chat");
  assert.equal(log.getAttribute("aria-live"), "polite");
  assert.equal(handle.getByRole("article", { name: "Ada" }).getAttribute("data-author"), "Ada");
  assert.equal(
    handle.getByRole("form", { name: "Composer" }).getAttribute("data-vize-ui"),
    "chat-composer",
  );
  handle.unmount();
});

test("sends trimmed text and clears the draft", async () => {
  const handle = mountInteraction(ChatComposer);
  const box = handle.getByRole("textbox", { name: "Message" }) as HTMLTextAreaElement;
  const send = handle.getByRole("button", { name: "Send" }) as HTMLButtonElement;

  assert.equal(send.disabled, true);
  await typeMessage(box, "  hi  ");
  assert.equal(send.disabled, false);

  const pressed = await handle.press(box, "Enter");
  assert.equal(pressed.keydownPrevented, true);
  assert.deepEqual(handle.wrapper.emitted("send")?.[0], ["hi"]);
  assert.equal(box.value, "");
  assert.equal(send.disabled, true);
  handle.unmount();
});

test("keeps Shift+Enter as a newline", async () => {
  const handle = mountInteraction(ChatComposer);
  const box = handle.getByRole("textbox", { name: "Message" }) as HTMLTextAreaElement;
  await typeMessage(box, "hi");
  box.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Enter", shiftKey: true, bubbles: true, cancelable: true }),
  );
  await nextTick();

  assert.equal(handle.wrapper.emitted("send"), undefined);
  assert.equal(box.value, "hi");
  handle.unmount();
});

test("a disabled composer does not send", async () => {
  const handle = mountInteraction(ChatComposer, { props: { disabled: true } });
  const box = handle.getByRole("textbox", { name: "Message" }) as HTMLTextAreaElement;
  await typeMessage(box, "hi");
  await handle.press(box, "Enter");

  assert.equal(handle.wrapper.emitted("send"), undefined);
  assert.equal((handle.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled, true);
  handle.unmount();
});
