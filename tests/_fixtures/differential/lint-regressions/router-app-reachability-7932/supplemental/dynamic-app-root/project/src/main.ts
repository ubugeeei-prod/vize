import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

const choose = () => App;
const app = createApp(choose());
app.use(router);
