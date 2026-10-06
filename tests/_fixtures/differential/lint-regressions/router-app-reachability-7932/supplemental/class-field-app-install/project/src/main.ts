import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

export const Factory = class { app = createApp(App).use(router); };
