import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

const app = createApp(App);
export function install(app: { use(plugin: unknown): void }): void {
  app.use(router);
}
app.mount("#app");
