import { createVaporApp } from "vue";
import "./monacoBootstrap";
import App from "./app.vue";
// Import vize component styles (extracted CSS in production)
import "virtual:vize-styles";

createVaporApp(App).mount("#app");
