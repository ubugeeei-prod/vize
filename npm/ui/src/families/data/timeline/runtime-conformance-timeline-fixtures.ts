import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  TimelineConnector,
  TimelineContent,
  TimelineIndicator,
  TimelineItem,
  TimelineRoot,
  TimelineTime,
} from "./timeline.ts";
import type { TimelineItemSlotState } from "./timeline-types.ts";

const describe = ({ index, status }: TimelineItemSlotState) => `${index}:${status ?? "none"}`;

function renderTimeline() {
  return h(
    TimelineRoot,
    { ariaLabel: "Deploys", value: "build" },
    {
      default: () =>
        ["commit", "build", "release"].map((value) =>
          h(
            TimelineItem,
            { key: value, value },
            {
              default: () => [
                h(TimelineIndicator, null, { default: describe }),
                h(TimelineConnector, null, { default: describe }),
                h(TimelineContent, null, { default: describe }),
                h(
                  TimelineTime,
                  { datetime: "2026-09-25" },
                  { default: ({ datetime }: { readonly datetime: string }) => datetime },
                ),
              ],
            },
          ),
        ),
    },
  );
}

function fixture(name: string, file: string, marker: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/data/timeline/${file}`,
    render: renderTimeline,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${marker}"`));
      assert.match(html, /aria-label="Deploys"/);
      assert.match(html, /aria-current="step"/);
      assert.match(html, /1:current/);
    },
    assertHydratedDom(host) {
      const element = host.querySelector(`[data-vize-ui="${marker}"]`);
      assert.ok(element instanceof HTMLElement);
      const current = host.querySelector('[aria-current="step"]');
      assert.ok(current instanceof HTMLLIElement);
      assert.equal(current.getAttribute("data-value"), "build");
    },
  };
}

export const timelineRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("timeline", "timeline-root.vue", "timeline"),
  fixture("timeline-item", "timeline-item.vue", "timeline-item"),
  fixture("timeline-indicator", "timeline-indicator.vue", "timeline-indicator"),
  fixture("timeline-connector", "timeline-connector.vue", "timeline-connector"),
  fixture("timeline-content", "timeline-content.vue", "timeline-content"),
  fixture("timeline-time", "timeline-time.vue", "timeline-time"),
];
