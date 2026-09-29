import type { App, Component } from "vue";

import {
  ClientOnly,
  NuxtClientFallback,
  NuxtErrorBoundary,
  NuxtImg,
  NuxtIsland,
  NuxtLayout,
  NuxtLink,
  NuxtLoadingIndicator,
  NuxtPage,
  NuxtPicture,
  NuxtRouteAnnouncer,
  NuxtWelcome,
  RouterLink,
  RouterView,
} from "./mocks/components.js";
import { useRoute, useRouter } from "./mocks/composables.js";
import { useNuxtApp, useRuntimeConfig } from "./mocks/runtime.js";
import { configureNuxtMuseaMocks } from "./context.js";
import type { NuxtMuseaOptions } from "./types.js";

const builtInComponents = {
  NuxtLink,
  NuxtPage,
  ClientOnly,
  NuxtLayout,
  NuxtLoadingIndicator,
  NuxtErrorBoundary,
  NuxtRouteAnnouncer,
  NuxtWelcome,
  NuxtIsland,
  NuxtClientFallback,
  NuxtImg,
  NuxtPicture,
};

export function installNuxtMuseaMocks(app: App, options: NuxtMuseaOptions = {}): App {
  configureNuxtMuseaMocks(options);

  for (const [name, component] of Object.entries(builtInComponents)) {
    app.component(name, component);
  }
  // vue-router registers these itself. Do not replace a component already on the app.
  registerUnlessPresent(app, "RouterLink", RouterLink);
  registerUnlessPresent(app, "RouterView", RouterView);

  const globals = app.config.globalProperties;
  globals.$config = useRuntimeConfig();
  // vue-router defines `$route` as a non-configurable getter; assigning throws.
  // Leave `$router` as well when a router already installed it.
  assignGlobalUnlessDefined(globals, "$route", useRoute());
  assignGlobalUnlessDefined(globals, "$router", useRouter());
  app.provide("nuxt-app", useNuxtApp());
  return app;
}

function registerUnlessPresent(app: App, name: string, component: Component): void {
  if (app.component(name)) return;
  app.component(name, component);
}

function assignGlobalUnlessDefined(target: object, key: string, value: unknown): void {
  const descriptor = Object.getOwnPropertyDescriptor(target, key);
  // vue-router's `$route` is a non-configurable getter; assigning it throws.
  // A router-provided `$router` is left in place as well.
  if (descriptor) return;
  (target as Record<string, unknown>)[key] = value;
}

export function createNuxtMuseaPreviewSetup(options: NuxtMuseaOptions = {}) {
  return (app: App) => {
    installNuxtMuseaMocks(app, options);
  };
}
