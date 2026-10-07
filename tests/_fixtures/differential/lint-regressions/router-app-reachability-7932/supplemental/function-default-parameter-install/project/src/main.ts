import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

export const factory = function(app = createApp(App).use(router)) { return app; };
