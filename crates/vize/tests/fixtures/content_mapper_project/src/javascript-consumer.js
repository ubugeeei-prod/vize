import Child from "./Child.vue";

/** @param {InstanceType<typeof Child>["$props"]} props */
export function readChildCount(props) {
  return props.count;
}

readChildCount({ count: 1 });
