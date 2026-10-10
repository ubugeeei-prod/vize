import assert from "node:assert/strict";
import type { Page } from "playwright";

export async function currentUnsupportedProps(page: Page) {
  return JSON.parse((await page.locator(".props-json-code code").textContent())!) as Record<
    string,
    string
  >;
}

export async function waitUnsupportedProps(page: Page, expected: Record<string, string>) {
  await page.waitForFunction(
    (serialized) => {
      return document.querySelector(".props-json-code code")?.textContent === serialized;
    },
    JSON.stringify(expected, null, 2),
  );
  assert.deepEqual(await currentUnsupportedProps(page), expected);
}

export function supportedView(values: Record<string, string>) {
  return {
    props: Object.fromEntries(Object.entries(values).filter(([name]) => name !== "__proto__")),
    attrs: {},
    ownProto: false,
    prototypeUnchanged: true,
  };
}

export async function waitUnsupportedPreview(page: Page, values: Record<string, string>) {
  await page.waitForFunction(
    (serialized) => {
      const frame = document.querySelector<HTMLIFrameElement>(".props-preview iframe");
      return frame?.contentDocument?.querySelector("output")?.textContent === serialized;
    },
    JSON.stringify(supportedView(values)),
  );
  return page.frameLocator(".props-preview iframe").locator("output").textContent();
}

export async function observeUnsupportedMessages(page: Page) {
  await page.frameLocator(".props-preview iframe").locator("output").waitFor();
  await page
    .frameLocator(".props-preview iframe")
    .locator("body")
    .evaluate(() => {
      Reflect.set(window, "__unsupportedPropMessages", []);
      window.addEventListener("message", (event) => {
        if (event.data?.type !== "musea:set-props") return;
        const props = event.data.payload.props;
        Reflect.get(window, "__unsupportedPropMessages").push({
          props,
          keys: Object.keys(props),
          ownProto: Object.hasOwn(props, "__proto__"),
          prototypeUnchanged: Object.getPrototypeOf(props) === Object.prototype,
        });
      });
    });
}

export async function unsupportedMessage(page: Page, expected: Record<string, string>) {
  await page.waitForFunction((serialized) => {
    const frame = document.querySelector<HTMLIFrameElement>(".props-preview iframe");
    const messages =
      frame?.contentWindow && Reflect.get(frame.contentWindow, "__unsupportedPropMessages");
    return messages?.length && JSON.stringify(messages.at(-1).props) === serialized;
  }, JSON.stringify(expected));
  const serialized = await page
    .frameLocator(".props-preview iframe")
    .locator("body")
    .evaluate(() => JSON.stringify(Reflect.get(window, "__unsupportedPropMessages").at(-1)));
  const message = JSON.parse(serialized);
  assert.deepEqual(message, {
    props: expected,
    keys: Object.keys(expected),
    ownProto: Object.hasOwn(expected, "__proto__"),
    prototypeUnchanged: true,
  });
  return message;
}

/** Keyboard input goes through the actual loaded Monaco editor and its change listener. */
export async function editUnsupportedJson(page: Page, values: Record<string, string>) {
  await page.locator(".props-mode-btn").filter({ hasText: "Code" }).click();
  const input = page.locator(".props-code-editor .monaco-editor textarea").first();
  await input.click();
  await input.press("ControlOrMeta+A");
  await page.keyboard.insertText(JSON.stringify(values, null, 2));
}

export async function addRawProp(page: Page, name: string, value: string) {
  await page.getByRole("button", { name: "Add Prop", exact: true }).click();
  await page.getByRole("textbox", { name: "Prop name", exact: true }).fill(name);
  await page.getByRole("textbox", { name: "Default value", exact: true }).fill(value);
  await page.getByRole("button", { name: "Add", exact: true }).click();
  assert.equal(await page.getByRole("textbox", { name, exact: true }).isEnabled(), true);
}

export async function rawEditorPrototype(page: Page) {
  const serialized = await page.locator(".props-json-code code").evaluate((node) => {
    const values = JSON.parse(node.textContent!);
    return JSON.stringify({
      values,
      keys: Object.keys(values),
      ownProto: Object.hasOwn(values, "__proto__"),
      prototypeUnchanged: Object.getPrototypeOf(values) === Object.prototype,
      globalPrototypeUnchanged:
        Object.getPrototypeOf(Object.prototype) === null && Object.prototype.constructor === Object,
    });
  });
  const observed = JSON.parse(serialized) as {
    values: Record<string, string>;
    keys: string[];
    ownProto: boolean;
    prototypeUnchanged: boolean;
    globalPrototypeUnchanged: boolean;
  };
  assert.equal(observed.prototypeUnchanged, true);
  assert.equal(observed.globalPrototypeUnchanged, true);
  return observed;
}
