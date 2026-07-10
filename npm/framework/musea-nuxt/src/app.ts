import type { App } from "vue";

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

  app.config.globalProperties.$config = useRuntimeConfig();
  app.config.globalProperties.$route = useRoute();
  app.config.globalProperties.$router = useRouter();
  app.provide("nuxt-app", useNuxtApp());
  return app;
}

export function createNuxtMuseaPreviewSetup(options: NuxtMuseaOptions = {}) {
  return (app: App) => {
    installNuxtMuseaMocks(app, options);
  };
}
