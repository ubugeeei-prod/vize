// `v-slots:default` is read as `v-slots`, as Babel does (vuetify, #9075).
import { h } from "vue";
const Box = (props, { slots }) => h("section", null, [slots.default?.()]);
const icon = () => h("i", null, "icon");
export const App = () => <Box v-slots:default={icon as () => unknown} />;
