# Inspect and profile Fresco applications

Fresco devtools are opt-in and dependency-free. Create one collector, pass it to the public
application options, and read immutable snapshots when a test, logger, or in-terminal overlay needs
them:

```ts
import { createApp, createFrescoDevtools, formatFrescoDebugOverlay } from "@vizejs/fresco";
import App from "./App.vue";

const devtools = createFrescoDevtools({ eventLimit: 200 });
const app = createApp(App, { devtools });

await app.mount();
console.error(formatFrescoDebugOverlay(devtools.snapshot()));
```

The snapshot joins stable host-node ids to the latest native `x`, `y`, `width`, and `height`
rectangles. It also reports unmatched native ids, so a stale or synthetic layout cannot silently look
like a live Vue node. Host props are shallow-copied and parent back-references are replaced with
`parentId`/`childIds`, making the tree safe to index without retaining the mutable renderer graph.

## Event trace

The trace is a closed union of `create`, `update`, `resize`, `remove`, `input`, and `render` records.
Every record receives a collector-local monotonic sequence number. `eventLimit` bounds retained
memory and discards the oldest records first; set it to `0` when only tree and metrics snapshots are
needed. `clearEvents()` clears the trace without losing the latest tree or render metrics.

Input records contain Fresco's normalized key, mouse, paste, resize, focus, and composition events,
not backend-specific terminal payloads. Render records include elapsed time plus host-node and native
layout counts.

## Layout assertions without a terminal

`inspectFrescoTree(root, layouts)` is a pure helper. Tests can pass a tree mounted through
`@vizejs/fresco/testing` and either recorded native layouts or an explicit fixture:

```ts
const inspection = inspectFrescoTree(rendered.root, recordedLayouts);
const save = inspection.nodes.find((node) => node.props["aria-label"] === "Save");

assert.deepEqual(save?.layout, { x: 2, y: 4, width: 12, height: 1 });
```

`formatFrescoDebugOverlay()` produces compact terminal text rather than writing to a stream. Render
that text through Fresco's ordinary `Text`, `FrameBuffer`, or overlay composition, or send it to a
separate diagnostic stream. This keeps inspection optional and prevents devtools from scheduling
frames while the application is idle.
