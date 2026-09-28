// 🍣 Spread-child regression #6888.
import { read, enabled, rows } from "./spread-child-state.mjs";
export const App = () => <main><section><i>before</i>{...read()}<b>after</b></section><aside>untouched</aside></main>;
