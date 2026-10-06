import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

const app = createApp(App);
Math.random() > 0.5 && app.use(router);
