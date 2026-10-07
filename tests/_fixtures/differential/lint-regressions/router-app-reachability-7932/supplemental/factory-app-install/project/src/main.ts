import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

export function bootstrap() {
  const app = createApp(App);
  app.use(router);
  return app;
}
