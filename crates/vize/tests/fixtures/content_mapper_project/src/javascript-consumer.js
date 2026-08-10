import Child from "./Child.vue";

/** @param {InstanceType<typeof Child>["$props"]} props */
export function readChildCount(props) {
  return props.count;
}

readChildCount({ count: 1 });

// `Child` is only referenced from the JSDoc type above, which the repo linter
// does not count as a use; keep a value reference so the fixture stays lint-clean.
void Child;
