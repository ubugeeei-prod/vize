import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import { router as secondRouter } from "./second-router";

const app = createApp(App);
app.use(router);
app.use(secondRouter);
