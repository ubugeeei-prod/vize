// A sole object child whose entries are references (naive-ui, #9075).
import { h } from "vue";
const Box = (props, { slots }) => h("section", null, [slots.title?.(), slots.default?.()]);
const icon = () => h("i", null, "icon");
export const App = () => <Box>{{ default: icon, title: () => "t" }}</Box>;
