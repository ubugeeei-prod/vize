---
title: Global Preview Controls
description: Share brand, component theme, locale and other project settings across Musea previews.
---

# Global Preview Controls

Add project controls to Musea's addon toolbar with `toolbar`. A selection reaches every mounted
preview, including the variant grid, props panel, accessibility panel, fullscreen view and multiple
viewports. The Vue apps update in place; changing a global does not reload the iframe.

## Define controls

```ts
import { musea } from "@vizejs/vite-plugin-musea";

musea({
  previewSetup: "musea.preview.ts",
  toolbar: [
    {
      id: "brand",
      title: "Brand",
      type: "select",
      options: ["default", "brand-a", "brand-b"],
      default: "default",
    },
    {
      id: "scheme",
      title: "Component theme",
      type: "toggle",
      options: [
        { value: "light", label: "Light", icon: "☀" },
        { value: "dark", label: "Dark", icon: "☾" },
      ],
      default: "light",
    },
    {
      id: "locale",
      title: "Locale",
      type: "select",
      options: [
        { value: "en", label: "English" },
        { value: "ja", label: "日本語" },
      ],
      default: "en",
    },
  ],
});
```

| Field     | Contract                                                                                                                    |
| --------- | --------------------------------------------------------------------------------------------------------------------------- |
| `id`      | Unique key starting with a letter; subsequent characters may be letters, numbers, `_` or `-`.                               |
| `title`   | Visible label and accessible name.                                                                                          |
| `type`    | `select` renders a dropdown; `toggle` renders two explicitly labeled buttons.                                               |
| `options` | Nonempty array of strings or `{ value, label, icon? }`. Values are strings, finite numbers or booleans, and must be unique. |
| `default` | An option's exact value, preserving its primitive type.                                                                     |
| `icon`    | Optional text, such as a Unicode symbol. HTML and SVG strings are displayed as text.                                        |

Invalid configuration fails when the plugin loads. Controls are supported with Vue 3 and Vue 2.7.
Omitting `toolbar` preserves existing preview behavior. The built-in Light/Dark background buttons
continue to set the canvas background independently of your component's theme.

## Apply values in the preview

With nonempty toolbar controls, the setup module receives a context containing a stable Vue ref.
Read its initial values before mounting, or watch the ref to react to subsequent toolbar changes.
Without controls, context is omitted. Guard the optional argument when reusing a setup module
across configurations. Existing setup functions that accept only `app` remain compatible.

```ts
// musea.preview.ts
import { watch, type App } from "vue";
import type { MuseaPreviewContext } from "@vizejs/vite-plugin-musea";
import { createI18n } from "vue-i18n";

export default function setup(app: App, context?: MuseaPreviewContext) {
  const i18n = createI18n({
    legacy: false,
    locale: context?.globals.value.locale === "ja" ? "ja" : "en",
    messages: { en: {}, ja: {} },
  });
  app.use(i18n);

  if (!context) return;
  const { globals } = context;

  const stop = watch(
    globals,
    (values) => {
      document.documentElement.dataset.brand = String(values.brand);
      document.documentElement.dataset.scheme = String(values.scheme);
      i18n.global.locale.value = values.locale === "ja" ? "ja" : "en";
    },
    { immediate: true },
  );
  app.onUnmount(stop);
}
```

Use your own theme API in the watcher to update Vuetify, CSS custom properties, density or direction.
In Vue 2.7, release an external watcher with `app.$on("hook:destroyed", stop)` instead of
`app.onUnmount(stop)`. Each iframe owns its ref; Musea delivers the same validated values to all of them.
Prop-driven app remounts reuse that iframe's current ref.

## Share and restore state

Musea stores global values in local storage under the gallery's base path and preserves them during
SPA navigation. The `museaGlobals` URL query contains their JSON representation, so sharing the
current gallery URL reproduces the controls in a fresh browser. Explicit URL values take precedence
over stored values. Unknown keys are discarded; unsupported option values fall back to the configured
default. Other query parameters and the URL hash are preserved.

Storage is optional. If the browser disables local storage, toolbar updates, navigation and shared
URLs still work. The same controls and preview setup are included in a built static gallery.

Try the working brand/theme/locale example with `pnpm --dir examples/vite-musea gallery:globals`.
The default example command retains its existing gallery configuration.

Automatic VRT capture across a Cartesian product of globals is not implemented. Existing VRT runs
use the configured defaults; do not assume that adding controls creates extra screenshot cases.
