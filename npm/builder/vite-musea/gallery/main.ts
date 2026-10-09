import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import { initializeGlobals } from "./composables/useGlobals";
import "./styles/gallery.css";
import "highlight.js/styles/github-dark.css";
import "./styles/hljs-light.css";

const app = createApp(App);
initializeGlobals(router);
app.use(router);
app.mount("#app");
import "./styles/code.css";
