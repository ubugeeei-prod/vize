// 🍣 Spread-child regression #6888.
import { read, enabled, rows } from "./spread-child-state.mjs";
export const App = () => <main>{rows.map(row => <div key={row.id}>{...read(row.values)}</div>)}</main>;
