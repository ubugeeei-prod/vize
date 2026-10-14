import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick, ref } from "vue";

import FullscreenButton from "../../actions/fullscreen-button/fullscreen-button.vue";
import type { FullscreenButtonController } from "../../actions/fullscreen-button/fullscreen-button-types.ts";
import PanZoomContent from "../pan-zoom/pan-zoom-content.vue";
import PanZoomRoot from "../pan-zoom/pan-zoom-root.vue";
import PanZoomViewport from "../pan-zoom/pan-zoom-viewport.vue";
import type { LightboxSlotState } from "./lightbox.ts";
import LightboxContent from "./lightbox-content.vue";
import LightboxImage from "./lightbox-image.vue";
import LightboxItem from "./lightbox-item.vue";
import LightboxRoot from "./lightbox-root.vue";
import LightboxTrigger from "./lightbox-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface Photo {
  readonly src: string;
  readonly alt: string;
}

const photos: readonly Photo[] = ["Harbor", "Market"].map((name) => ({
  src: `/photos/${name.toLowerCase()}.jpg`,
  alt: name,
}));

function mountPreview(controller?: FullscreenButtonController) {
  const stage = ref<HTMLElement | null>(null);
  const handle = mountInteraction(LightboxRoot, {
    props: { id: "preview", items: photos },
    record: ["update:index"],
    slots: {
      default: (state: LightboxSlotState<Photo>) => [
        h(LightboxTrigger, { index: 0 }, () => "Open Harbor"),
        h(LightboxContent, { portalDisabled: true }, () =>
          h("div", { ref: stage, "data-stage": "" }, [
            h(LightboxItem, null, () =>
              state.item === undefined
                ? null
                : h(PanZoomRoot, { key: state.item.src, maxScale: 4 }, () =>
                    h(PanZoomViewport, { ariaLabel: `Zoom ${state.item?.alt}` }, () =>
                      h(PanZoomContent, null, () =>
                        h(LightboxImage, { src: state.item?.src, alt: state.item?.alt }),
                      ),
                    ),
                  ),
            ),
            controller === undefined
              ? null
              : h(FullscreenButton, { controller, target: () => stage.value }),
          ]),
        ),
      ],
    },
  });
  return { handle, stage };
}

function part(root: ParentNode, name: string): HTMLElement {
  const element = root.querySelector<HTMLElement>(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function key(target: HTMLElement, value: string): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

async function open(root: HTMLElement): Promise<void> {
  part(root, "lightbox-trigger").click();
  await nextTick();
  await nextTick();
}

test("zoom keys on a focused slide pan and zoom the image instead of changing slides", async () => {
  const { handle } = mountPreview();
  const root = handle.root().ownerDocument.body;
  await open(root);
  const viewport = part(root, "pan-zoom-viewport");
  const content = part(root, "pan-zoom-content");
  viewport.focus();

  const pan = key(viewport, "ArrowRight");
  await nextTick();
  assert.equal(pan.defaultPrevented, true, "the zoom viewport owns the arrow key");
  assert.notEqual(content.style.getPropertyValue("--vize-ui-pan-zoom-x"), "0px");
  key(viewport, "+");
  await nextTick();
  assert.ok(Number(content.style.getPropertyValue("--vize-ui-pan-zoom-scale")) > 1);
  assert.deepEqual(handle.recorded(), [], "panning and zooming never change the slide");

  key(part(root, "lightbox-content"), "ArrowRight");
  await nextTick();
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [["update:index", 1]],
    "arrow keys outside the zoom viewport still change slides",
  );
  handle.unmount();
});

test("a fullscreen button inside the viewer targets the open lightbox stage", async () => {
  const requested: Element[] = [];
  const controller: FullscreenButtonController = {
    getFullscreenElement: () => null,
    requestFullscreen: (target) => {
      requested.push(target);
    },
    exitFullscreen: () => {},
  };
  const { handle, stage } = mountPreview(controller);
  const root = handle.root().ownerDocument.body;
  await open(root);
  part(root, "fullscreen-button").click();
  await nextTick();
  await nextTick();
  assert.ok(stage.value);
  assert.deepEqual(requested, [stage.value]);
  assert.ok(part(root, "lightbox-content").contains(stage.value));
  handle.unmount();
});
