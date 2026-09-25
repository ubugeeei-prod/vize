import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  SignaturePadCanvas,
  SignaturePadClear,
  SignaturePadGuide,
  SignaturePadRedo,
  SignaturePadRoot,
  SignaturePadUndo,
} from "./signature-pad.ts";

const strokes = [{ points: [{ x: 4, y: 4, pressure: 0.5, time: 0 }] }];

function pad(id: string) {
  return h(SignaturePadRoot, { id, defaultValue: strokes, width: 120, height: 60 }, () => [
    h(SignaturePadGuide, null, () => "Sign here"),
    h(SignaturePadCanvas, { ariaLabel: "Customer signature" }),
    h(SignaturePadUndo, null, () => "Undo"),
    h(SignaturePadRedo, null, () => "Redo"),
    h(SignaturePadClear, null, () => "Clear"),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const signaturePadRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "signature-pad",
    sourceFile: "families/media/signature-pad/signature-pad-root.vue",
    render: () => pad("contract-signature"),
    assertServerMarkup(html) {
      assert.match(html, /<div id="contract-signature" data-vize-ui="signature-pad-root"/);
      assert.match(html, /data-state="filled"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "signature-pad-root").id, "contract-signature");
    },
  },
  {
    name: "signature-pad-canvas",
    sourceFile: "families/media/signature-pad/signature-pad-canvas.vue",
    render: () => pad("signature-canvas"),
    assertServerMarkup(html) {
      assert.match(html, /role="img" aria-label="Customer signature" viewBox="0 0 120 60"/);
      assert.match(html, /data-vize-ui="signature-pad-stroke"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "signature-pad-canvas") instanceof SVGSVGElement);
    },
  },
  {
    name: "signature-pad-guide",
    sourceFile: "families/media/signature-pad/signature-pad-guide.vue",
    render: () => pad("signature-guide"),
    assertServerMarkup(html) {
      assert.match(html, /aria-hidden="true" data-vize-ui="signature-pad-guide"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "signature-pad-guide").textContent, "Sign here");
    },
  },
  {
    name: "signature-pad-undo",
    sourceFile: "families/media/signature-pad/signature-pad-undo.vue",
    render: () => pad("signature-undo"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /<button type="button" disabled aria-controls="signature-undo" data-vize-ui="signature-pad-undo"/,
      );
    },
    assertHydratedDom(host) {
      const button = part(host, "signature-pad-undo");
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.disabled, true);
    },
  },
  {
    name: "signature-pad-redo",
    sourceFile: "families/media/signature-pad/signature-pad-redo.vue",
    render: () => pad("signature-redo"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="signature-pad-redo"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "signature-pad-redo") instanceof HTMLButtonElement);
    },
  },
  {
    name: "signature-pad-clear",
    sourceFile: "families/media/signature-pad/signature-pad-clear.vue",
    render: () => pad("signature-clear"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /<button type="button" aria-controls="signature-clear" data-vize-ui="signature-pad-clear"/,
      );
    },
    assertHydratedDom(host) {
      const button = part(host, "signature-pad-clear");
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.disabled, false);
    },
  },
];
