import { createDevtoolsTraceRecorder } from "./index.ts";
import type { DevtoolsEvent, DevtoolsSourceSpecifier } from "./types.ts";

const recorder = createDevtoolsTraceRecorder();
recorder.record({
  kind: "render:end",
  componentId: "article",
  componentName: "ArticleView",
  source: "src/pages/ArticleView.vue",
  durationMs: 1,
});

const source: DevtoolsSourceSpecifier = "src/App.vue";
const event = {
  kind: "reactivity:trigger",
  sourceId: "ref:count",
  effectId: "effect:render",
  operation: "set",
  source,
} satisfies DevtoolsEvent;
recorder.record(event);

// @ts-expect-error Devtools sources must stay owned by .vue files.
const _invalidSource: DevtoolsSourceSpecifier = "src/App.ts";

recorder.record({
  kind: "render:start",
  componentId: "broken",
  componentName: "Broken",
  // @ts-expect-error Render traces must point at authored .vue files.
  source: "src/Broken.ts",
});
