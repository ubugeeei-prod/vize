import { createVaporApp } from "vue";
import "./monacoBootstrap";
import App from "./App.vue";
import "./styles.css";
// Import vize component styles (extracted CSS in production)
import "virtual:vize-styles";

createVaporApp(App).mount("#app");
