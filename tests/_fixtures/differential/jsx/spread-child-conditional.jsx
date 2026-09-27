// 🍣 Spread-child regression #6888.
import { read, enabled, rows } from "./spread-child-state.mjs";
export const App = () => <div>{enabled() ? <>{...read()}</> : <i>off</i>}</div>;
