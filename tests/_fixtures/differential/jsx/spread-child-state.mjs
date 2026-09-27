// Runtime instrumentation imported by the authored spread-child programs.
import { shallowRef, shallowReactive } from "vue";

export const state = shallowRef([]);
export const flag = shallowRef(true);
export const stats = { reads: 0, iterations: 0 };
export function read(...args) {
  stats.reads += 1;
  return args.length ? args[0] : state.value;
}
export function enabled() {
  return flag.value;
}
export const rows = shallowReactive([
  { id: "row", get values() { return state.value; } },
]);
