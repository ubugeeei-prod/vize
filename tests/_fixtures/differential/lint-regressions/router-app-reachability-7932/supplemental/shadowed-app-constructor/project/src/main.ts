import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

export function install(createApp: (root: unknown) => { use(plugin: unknown): void }): void {
  const app = createApp(App);
  app.use(router);
}
