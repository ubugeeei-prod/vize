// A slots object built from a spread (vuetify, #9075).
import { h } from "vue";
const Box = (props, { slots }) => h("section", null, [slots.title?.(), slots.default?.()]);
const icon = () => h("i", null, "icon");
const shared = { title: () => "t" };
export const App = () => <Box v-slots={{ ...shared, default: icon }} />;
