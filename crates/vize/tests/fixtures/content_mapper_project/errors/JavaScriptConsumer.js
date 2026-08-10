import Child from "../src/Child.vue";

/** @type {InstanceType<typeof Child>["$props"]} */
const props = { count: "wrong" };

void props;
