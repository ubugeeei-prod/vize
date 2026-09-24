import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import { h, nextTick } from "vue";

import { createQrCodeNumericSegment, encodeQrCode, qrCodeToSvgPath } from "./qr-code.ts";
import { qrCodeKanjiEncoder } from "./qr-code-kanji.ts";
import type { QrCodeExpose, QrCodeSlotState } from "./qr-code.ts";
import QrCode from "./qr-code.vue";

interface QrHandle {
  readonly wrapper: ReturnType<typeof mount>;
  readonly root: () => Element;
  readonly exposes: <Exposed>() => Exposed;
  readonly unmount: () => void;
}

/** Mount into the live document; the SVG root is not an HTMLElement, so the HTML harness does not apply. */
function mountQrCode(options: {
  readonly props: Record<string, unknown>;
  readonly slots?: Record<string, unknown>;
}): QrHandle {
  const host = document.createElement("div");
  document.body.append(host);
  const wrapper = mount(QrCode, {
    attachTo: host,
    props: options.props,
    slots: options.slots,
  });
  return {
    wrapper,
    root: () => wrapper.element,
    exposes: <Exposed>() => wrapper.vm as unknown as Exposed,
    unmount: () => {
      wrapper.unmount();
      host.remove();
    },
  };
}

function imageNamed(root: Element, name: string): SVGSVGElement {
  assert.ok(root instanceof SVGSVGElement);
  assert.equal(root.getAttribute("role"), "img");
  assert.equal(root.getAttribute("aria-label"), name);
  return root;
}

test("renders an accessible SVG symbol with a quiet zone and data hooks", () => {
  const handle = mountQrCode({ props: { value: "https://vizejs.dev" } });
  const svg = imageNamed(handle.root(), "https://vizejs.dev");
  const matrix = encodeQrCode("https://vizejs.dev");
  const dimension = matrix.size + 8;

  assert.ok(svg instanceof SVGSVGElement);
  assert.equal(svg.getAttribute("data-vize-ui"), "qr-code");
  assert.equal(svg.getAttribute("part"), "root");
  assert.equal(svg.getAttribute("data-state"), "ready");
  assert.equal(svg.getAttribute("viewBox"), `0 0 ${dimension} ${dimension}`);
  assert.equal(svg.getAttribute("shape-rendering"), "crispEdges");
  assert.equal(svg.getAttribute("data-version"), String(matrix.version));
  assert.equal(svg.getAttribute("data-error-correction"), "M");
  assert.equal(svg.getAttribute("data-mask"), String(matrix.mask));
  assert.equal(svg.getAttribute("data-mode"), "byte");
  assert.equal(svg.getAttribute("data-size"), String(matrix.size));
  assert.equal(svg.querySelector("title")?.textContent, "https://vizejs.dev");
  const background = svg.querySelector('[part="background"]');
  assert.equal(background?.getAttribute("width"), String(dimension));
  assert.equal(background?.getAttribute("fill"), "none");
  const modules = svg.querySelector('[part="modules"]');
  assert.equal(modules?.getAttribute("fill"), "currentColor");
  assert.equal(modules?.getAttribute("d"), qrCodeToSvgPath(matrix, { quietZone: 4 }));
  handle.unmount();
});

test("forwards encoder options and re-encodes when props change", async () => {
  const handle = mountQrCode({
    props: { value: "HELLO", errorCorrection: "L", version: 5, mask: 3, quietZone: 0 },
  });
  const svg = handle.root();
  assert.equal(svg.getAttribute("data-version"), "5");
  assert.equal(svg.getAttribute("data-mask"), "3");
  assert.equal(svg.getAttribute("data-error-correction"), "L");
  assert.equal(svg.getAttribute("data-mode"), "alphanumeric");
  assert.equal(svg.getAttribute("viewBox"), "0 0 37 37");

  await handle.wrapper.setProps({
    boostErrorCorrection: true,
    mask: "auto",
    mode: "byte",
    value: "HELLO",
    version: "auto",
  });
  await nextTick();
  assert.equal(svg.getAttribute("data-version"), "1");
  assert.equal(svg.getAttribute("data-error-correction"), "H");
  assert.equal(svg.getAttribute("data-mode"), "byte");
  handle.unmount();
});

test("forwards segmentation, Kanji, and ECI options and accepts explicit segments", async () => {
  const url = "HTTPS://EXAMPLE.COM/ORDER/31415926535897932384626";
  const handle = mountQrCode({ props: { value: url, quietZone: 0 } });
  const svg = handle.root();
  assert.equal(svg.getAttribute("data-mode"), "mixed");
  const pathOf = (options: Parameters<typeof encodeQrCode>[1]) =>
    qrCodeToSvgPath(encodeQrCode(url, options));
  assert.equal(svg.querySelector("path")?.getAttribute("d"), pathOf({}));

  await handle.wrapper.setProps({ segmentation: "single" });
  assert.equal(svg.getAttribute("data-mode"), "alphanumeric");

  await handle.wrapper.setProps({
    value: "日本語",
    segmentation: "optimal",
    kanji: qrCodeKanjiEncoder,
  });
  assert.equal(svg.getAttribute("data-mode"), "kanji");
  assert.equal(svg.getAttribute("aria-label"), "日本語");

  await handle.wrapper.setProps({ kanji: undefined, utf8Eci: true });
  const exposed = handle.exposes<QrCodeExpose>();
  assert.equal(exposed.matrix?.eci, 26);
  assert.equal(svg.getAttribute("data-mode"), "byte");

  await handle.wrapper.setProps({ utf8Eci: false, eci: 3 });
  assert.equal(exposed.matrix?.eci, 3);

  await handle.wrapper.setProps({
    eci: undefined,
    value: [createQrCodeNumericSegment("2024")],
    label: "Year",
  });
  assert.equal(svg.getAttribute("data-mode"), "numeric");
  assert.equal(svg.getAttribute("aria-label"), "Year");
  handle.unmount();
});

test("supports explicit labels, byte values, decorative symbols, and colors", () => {
  const labelled = mountQrCode({
    props: {
      value: new Uint8Array([1, 2, 3]),
      label: "Pairing code",
      foreground: "#111",
      background: "#fff",
    },
  });
  const svg = imageNamed(labelled.root(), "Pairing code");
  assert.equal(svg.querySelector('[part="modules"]')?.getAttribute("fill"), "#111");
  assert.equal(svg.querySelector('[part="background"]')?.getAttribute("fill"), "#fff");
  labelled.unmount();

  const decorative = mountQrCode({ props: { value: "hidden", decorative: true } });
  const root = decorative.root();
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.querySelector("title"), null);
  decorative.unmount();
});

test("renders the overlay slot inside the SVG with module coordinates", () => {
  let seen: QrCodeSlotState | undefined;
  const handle = mountQrCode({
    props: { value: "logo", errorCorrection: "H" },
    slots: {
      overlay: (state: QrCodeSlotState) => {
        seen = state;
        return h("rect", { "data-logo": "", x: state.dimension / 2 - 2, width: 4 });
      },
    },
  });
  const logo = handle.root().querySelector("[data-logo]");
  assert.ok(logo instanceof SVGElement);
  assert.ok(logo.parentElement === handle.root());
  assert.equal(seen?.state, "ready");
  assert.equal(seen?.quietZone, 4);
  assert.equal(seen?.dimension, (seen?.matrix?.size ?? 0) + 8);
  assert.equal(seen?.matrix?.errorCorrection, "H");
  handle.unmount();
});

test("renders the fallback slot with a typed diagnostic when encoding fails", async () => {
  const handle = mountQrCode({
    props: { value: "x".repeat(40), version: 1 },
    slots: {
      fallback: (state: QrCodeSlotState) => h("em", null, state.error?.code ?? "none"),
    },
  });
  const root = handle.root();
  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("data-vize-ui"), "qr-code");
  assert.equal(root.getAttribute("data-state"), "error");
  assert.equal(root.getAttribute("data-error"), "VIZE_UI_QR_DATA_TOO_LONG");
  assert.equal(root.textContent, "VIZE_UI_QR_DATA_TOO_LONG");

  await handle.wrapper.setProps({ quietZone: -1, version: "auto" });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-error"), "VIZE_UI_QR_INVALID_OPTION");

  await handle.wrapper.setProps({ quietZone: 4 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "ready");
  handle.unmount();
});

test("exposes the encoded matrix, state, and element", async () => {
  const handle = mountQrCode({ props: { value: "01234567", quietZone: 2 } });
  const exposed = handle.exposes<QrCodeExpose>();
  assert.equal(exposed.state, "ready");
  assert.ok(exposed.element === handle.root());
  assert.deepEqual(exposed.matrix, encodeQrCode("01234567"));
  assert.equal(exposed.quietZone, 2);
  assert.equal(exposed.dimension, 25);
  assert.equal(exposed.error, null);

  await handle.wrapper.setProps({ value: "abc", mode: "numeric" });
  await nextTick();
  assert.equal(exposed.state, "error");
  assert.equal(exposed.matrix, null);
  assert.equal(exposed.element, null);
  assert.equal(exposed.error?.code, "VIZE_UI_QR_INVALID_MODE");
  handle.unmount();
});
