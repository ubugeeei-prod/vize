import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

const app = createApp(App);
const choose = () => router;
app.use(choose());
