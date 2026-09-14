# @vizejs/devtools

Typed, SSR-safe trace foundations for Vize devtools.

This package currently ships the first trace protocol slice for render/update events, reactive
edges, provide/inject ownership, and Suspense branch state. It is request-local by default: create a
recorder per server render or client app instance, then serialize the snapshot into a devtools panel
or generator artifact.

```ts
import { createDevtoolsTraceRecorder } from "@vizejs/devtools";

const trace = createDevtoolsTraceRecorder({ sessionId: "request-1" });
trace.record({
  kind: "render:end",
  componentId: "ArticleView",
  componentName: "ArticleView",
  source: "src/pages/ArticleView.vue",
  durationMs: 2.1,
});

const snapshot = trace.snapshot();
```

The optional `@vizejs/devtools/panel` entry exports a `.vue` panel component for rendering the
snapshot in host applications.
