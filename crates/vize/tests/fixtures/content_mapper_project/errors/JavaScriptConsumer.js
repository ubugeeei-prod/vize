import Child from "../src/Child.vue";

/** @type {InstanceType<typeof Child>["$props"]} */
const props = { count: "wrong" };

void props;
// `Child` is only referenced from the JSDoc type above, which the repo linter
// does not count as a use; keep a value reference so the fixture stays lint-clean.
void Child;
