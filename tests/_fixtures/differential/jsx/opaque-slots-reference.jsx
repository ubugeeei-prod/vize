// Slot object entries that are function references (vant, #9075).
import { h } from "vue";
const Box = (props, { slots }) => h("section", null, [slots.title?.(), slots.default?.()]);
const icon = () => h("i", null, "icon");
export const App = () => <Box v-slots={{ default: icon, title: () => "t" }} />;
