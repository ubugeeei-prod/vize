import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  LightboxClose,
  LightboxContent,
  LightboxCounter,
  LightboxImage,
  LightboxItem,
  LightboxNext,
  LightboxPrevious,
  LightboxRoot,
  LightboxThumbnail,
  LightboxThumbnails,
  LightboxTrigger,
} from "./lightbox.ts";
import type { LightboxSlotState } from "./lightbox.ts";

const photos = ["/gallery/one.jpg", "/gallery/two.jpg"];

function lightbox(id: string) {
  return h(
    LightboxRoot,
    { id, items: photos, defaultOpen: true },
    {
      default: (state: LightboxSlotState<string>) => [
        h(LightboxTrigger, { index: 0 }, () => "Open gallery"),
        h(LightboxContent, { portalDisabled: true }, () => [
          h(LightboxItem, null, () =>
            h(LightboxImage, { src: state.item ?? "", alt: "Gallery photo" }),
          ),
          h(LightboxPrevious),
          h(LightboxNext),
          h(LightboxClose),
          h(LightboxCounter),
          h(LightboxThumbnails, null, () => [
            h(LightboxThumbnail, { index: 0 }),
            h(LightboxThumbnail, { index: 1 }),
          ]),
        ]),
      ],
    },
  );
}

function fixture(name: string, file: string, serverPattern: RegExp, part: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/media/lightbox/${file}`,
    render: () => lightbox(name),
    assertServerMarkup(html) {
      assert.match(html, serverPattern);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(`[data-vize-ui="${part}"]`), `${part} must hydrate`);
    },
  };
}

export const lightboxRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture(
    "lightbox",
    "lightbox-root.vue",
    /data-vize-ui="lightbox-root"[^>]*data-state="open"/,
    "lightbox-root",
  ),
  fixture("lightbox-trigger", "lightbox-trigger.vue", /aria-haspopup="dialog"/, "lightbox-trigger"),
  fixture(
    "lightbox-content",
    "lightbox-content.vue",
    /aria-label="Media viewer"/,
    "lightbox-content",
  ),
  fixture(
    "lightbox-item",
    "lightbox-item.vue",
    /aria-roledescription="slide" aria-label="1 of 2"/,
    "lightbox-item",
  ),
  fixture("lightbox-image", "lightbox-image.vue", /src="\/gallery\/one\.jpg"/, "image-root"),
  fixture(
    "lightbox-previous",
    "lightbox-previous.vue",
    /aria-label="Previous item"/,
    "lightbox-previous",
  ),
  fixture("lightbox-next", "lightbox-next.vue", /aria-label="Next item"/, "lightbox-next"),
  fixture("lightbox-close", "lightbox-close.vue", /aria-label="Close"/, "lightbox-close"),
  fixture("lightbox-counter", "lightbox-counter.vue", />1 of 2</, "lightbox-counter"),
  fixture(
    "lightbox-thumbnails",
    "lightbox-thumbnails.vue",
    /role="tablist"/,
    "lightbox-thumbnails",
  ),
  fixture(
    "lightbox-thumbnail",
    "lightbox-thumbnail.vue",
    /role="tab"[^>]*aria-selected="true"/,
    "lightbox-thumbnail",
  ),
];
