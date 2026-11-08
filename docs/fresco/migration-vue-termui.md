# Migrate a Vue TermUI application to Fresco

Both projects author terminal interfaces with Vue, so most application state and SFC structure can
remain. The migration boundary is the renderer, component imports, terminal lifecycle, and exact
input behavior.

## Move the entrypoint

Use Fresco's public application lifecycle and keep the root SFC unchanged initially:

```ts
import { createApp } from "@vizejs/fresco";
import App from "./App.vue";

const app = createApp(App, { exitOnCtrlC: true });
await app.mount();
await app.waitUntilExit();
```

Fresco acquires and restores the terminal through this application object. Avoid direct imports
from `@vizejs/fresco-native` unless implementing a version-locked platform adapter.

## Replace component imports

Import components and composables only from `@vizejs/fresco`. The closest mappings are:

| Vue TermUI  | Fresco                                   |
| ----------- | ---------------------------------------- |
| `Box`       | `Box`                                    |
| `Text`      | `Text`                                   |
| `Input`     | `TextInput`                              |
| `Textarea`  | `TextArea`                               |
| `Select`    | `Select`                                 |
| `Tabs`      | `Tabs`                                   |
| `ScrollBox` | `ScrollBox` with `ScrollBar`             |
| `useInput`  | `useInput`, `useKeyPress`, or host route |
| custom draw | `FrameBuffer`                            |

Fresco components are typed, but not every reference component owns every interaction yet. Read
the [compatibility matrix](./compatibility.md) before removing application-level keyboard or focus
logic.

## Preserve source-owned SFCs

```vue
<script setup lang="ts">
import { Box, FrameBuffer, Text } from "@vizejs/fresco";

const logo = ["╭──────╮", "│ Vize │", "╰──────╯"];
</script>

<template>
  <Box :gap="1" flex-direction="column">
    <Text :bold="true">Dashboard</Text>
    <FrameBuffer :lines="logo" fg="cyan" />
  </Box>
</template>
```

`FrameBuffer` is the supported extension boundary for precomposed terminal rows. It keeps custom
components outside private renderer imports and crosses the same validated native protocol as
built-in nodes.

## Theme and focus scopes

Use `provideFrescoTheme()` for a root theme or a nested subtree override. Consumers call
`useFrescoTheme()` and receive a resolved reactive theme. Focus registration supports stable
parent IDs, delegation, focus traps, and deterministic next/previous traversal.

## Verify the migration

1. Mount the converted SFC through Fresco's Vitest or plain Node test harness.
2. Snapshot both render nodes and the semantic tree where accessibility is relevant.
3. Exercise keyboard, paste, resize, focus, and mouse input explicitly.
4. Run the real terminal application on every supported platform and degraded terminal profile.
5. Keep the JS and native package versions aligned before release.

The repository's `npm/fresco/examples/sfc-app` fixture is the executable SFC reference.
