import Child from "./Child.vue";

export type ChildPropsFromTsx = InstanceType<typeof Child>["$props"];

const props: ChildPropsFromTsx = { count: 1 };
void props;
