/** Assert device and permission examples against the browser's own feature detection. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

const normalize = (text: string | null) => (text ?? "").replace(/\s+/g, " ").trim();

async function output(page: Page): Promise<string> {
  await page.waitForFunction(
    () => !(document.querySelector("output")?.textContent ?? "").includes("Checking"),
  );
  return normalize(await page.locator("output").first().textContent());
}

/** Each entry mirrors the composable's support check and the example's fallback text. */
const support: Record<string, { detect: () => boolean; unavailable: string }> = {
  "use-bluetooth": {
    detect: () => Reflect.has(navigator, "bluetooth"),
    unavailable: "Web Bluetooth is not available in this browser.",
  },
  "use-hid": {
    detect: () => Reflect.has(navigator, "hid"),
    unavailable: "WebHID is not available in this browser.",
  },
  "use-serial": {
    detect: () => Reflect.has(navigator, "serial"),
    unavailable: "Web Serial is not available in this browser.",
  },
  "use-usb": {
    detect: () => Reflect.has(navigator, "usb"),
    unavailable: "WebUSB is not available in this browser.",
  },
  "use-web-midi": {
    detect: () => typeof Reflect.get(navigator, "requestMIDIAccess") === "function",
    unavailable: "Web MIDI is not available in this browser.",
  },
  "use-gamepad": {
    detect: () => typeof Reflect.get(navigator, "getGamepads") === "function",
    unavailable: "The Gamepad API is not available in this browser.",
  },
  "use-payment-request": {
    detect: () => typeof Reflect.get(globalThis, "PaymentRequest") === "function",
    unavailable: "The Payment Request API is not available; show your regular checkout.",
  },
  "use-web-authn": {
    detect: () =>
      typeof Reflect.get(globalThis, "PublicKeyCredential") === "function" &&
      Reflect.has(navigator, "credentials"),
    unavailable: "WebAuthn is not available in this browser.",
  },
  "use-contact-picker": {
    detect: () =>
      typeof Reflect.get(Reflect.get(navigator, "contacts") ?? {}, "select") === "function",
    unavailable: "The Contact Picker is not available; type the addresses instead.",
  },
  "use-eye-dropper": {
    detect: () => typeof Reflect.get(globalThis, "EyeDropper") === "function",
    unavailable: "The EyeDropper API is not available; type a hex value instead.",
  },
  "use-barcode-detector": {
    detect: () => typeof Reflect.get(globalThis, "BarcodeDetector") === "function",
    unavailable: "The Barcode Detection API is not available in this browser.",
  },
  "use-file-system-access": {
    detect: () =>
      typeof Reflect.get(globalThis, "showOpenFilePicker") === "function" &&
      typeof Reflect.get(globalThis, "showSaveFilePicker") === "function",
    unavailable: "The File System Access API is not available; use a download instead.",
  },
  "use-fullscreen": {
    detect: () => typeof document.documentElement.requestFullscreen === "function",
    unavailable: "Fullscreen is not available here.",
  },
  "use-notification": {
    detect: () => Reflect.has(globalThis, "Notification"),
    unavailable: "Notifications are not available in this browser.",
  },
  "use-permission": {
    detect: () => Reflect.has(navigator, "permissions"),
    unavailable: "This browser cannot report permission states.",
  },
  "use-share": {
    detect: () => typeof Reflect.get(navigator, "share") === "function",
    unavailable: "No share sheet here; showing a link instead.",
  },
  "use-vibrate": {
    detect: () => typeof Reflect.get(navigator, "vibrate") === "function",
    unavailable: "Vibration is not available in this browser.",
  },
  "use-wake-lock": {
    detect: () => Reflect.has(navigator, "wakeLock"),
    unavailable: "Screen Wake Lock is not available in this browser.",
  },
  "use-screen-details": {
    detect: () => Reflect.has(globalThis, "getScreenDetails"),
    unavailable: "Window Management is not available; open the notes in a new tab.",
  },
  "use-display-media": {
    detect: () =>
      typeof Reflect.get(navigator.mediaDevices ?? {}, "getDisplayMedia") === "function",
    unavailable: "Screen capture is not available in this browser.",
  },
  "use-user-media": {
    detect: () => typeof Reflect.get(navigator.mediaDevices ?? {}, "getUserMedia") === "function",
    unavailable: "Camera access is not available in this browser.",
  },
  "use-speech-recognition": {
    detect: () =>
      typeof Reflect.get(globalThis, "SpeechRecognition") === "function" ||
      typeof Reflect.get(globalThis, "webkitSpeechRecognition") === "function",
    unavailable: "Speech recognition is not available in this browser.",
  },
  "use-speech-synthesis": {
    detect: () =>
      Reflect.has(globalThis, "speechSynthesis") &&
      Reflect.has(globalThis, "SpeechSynthesisUtterance"),
    unavailable: "Speech synthesis is not available in this browser.",
  },
  "use-service-worker": {
    detect: () => Reflect.has(navigator, "serviceWorker"),
    unavailable: "Service workers are not available in this browser.",
  },
  "use-push-subscription": {
    detect: () => Reflect.has(globalThis, "PushManager") && Reflect.has(navigator, "serviceWorker"),
    unavailable: "Push messages are not available in this browser.",
  },
  "use-builtin-ai": {
    detect: () => typeof Reflect.get(globalThis, "LanguageDetector") === "function",
    unavailable:
      "Built-in AI is not available in this browser; detect the language on your server instead.",
  },
};

export const deviceExampleNames = Object.keys(support);

export async function exerciseDeviceExample(page: Page, name: string): Promise<string[]> {
  const entry = support[name];
  if (!entry) throw new Error(`No device law for ${name}`);
  const supported = await page.evaluate(entry.detect);
  const text = await output(page);
  if (!supported) {
    assert.equal(text, entry.unavailable, `${name}: fallback message`);
    return ["fallback matches missing support"];
  }
  assert.notEqual(text, entry.unavailable, `${name}: supported browsers do not show the fallback`);
  switch (name) {
    case "use-permission": {
      const state = await page.evaluate(
        async () => (await navigator.permissions.query({ name: "geolocation" })).state,
      );
      assert.match(
        text,
        state === "granted" ? /allowed/ : state === "denied" ? /blocked/ : /we will ask/,
      );
      break;
    }
    case "use-notification":
      assert.equal(
        text,
        `Notification permission: ${await page.evaluate(() => Notification.permission)}`,
      );
      break;
    case "use-fullscreen":
      await page.getByRole("button", { name: "Present fullscreen" }).click();
      await page.waitForFunction(() => document.fullscreenElement !== null);
      assert.equal(
        normalize(await page.locator("output").textContent()),
        "Presenting in fullscreen.",
      );
      await page.getByRole("button", { name: "Exit fullscreen" }).click();
      await page.waitForFunction(() => document.fullscreenElement === null);
      break;
    case "use-vibrate":
      await page.getByRole("button", { name: "Buzz twice" }).click();
      assert.match(
        normalize(await page.locator("output").textContent()),
        /^(Vibration requested\.|The device declined to vibrate\.)$/,
      );
      break;
    case "use-web-authn":
      await page.getByRole("button", { name: "Check for passkey support" }).click();
      await page.waitForFunction(
        () => !document.querySelector("output")?.textContent?.includes("WebAuthn is available."),
      );
      assert.match(normalize(await page.locator("output").textContent()), /passkey|authenticator/);
      break;
    case "use-gamepad":
      assert.equal(text, "Connect a controller and press any button.");
      break;
    case "use-share":
      assert.equal(await page.getByRole("button", { name: "Share Vize" }).count(), 1);
      break;
    default:
      break;
  }
  return ["output follows the browser's support"];
}
