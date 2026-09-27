// 🍣 Spread-child regression #6888.
import { read, enabled, rows } from "./spread-child-state.mjs";
import { h } from "vue";
const Box = (props, { slots }) => h("section", null, slots.default());
export const App = () => <Box><i>before</i>{...read()}<b>after</b></Box>;
