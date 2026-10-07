import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

export const factory = (app = createApp(App).use(router)) => app;
