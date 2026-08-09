import App from "./App.vue";
import Public from "./Public.vue";

type IsAny<T> = 0 extends 1 & T ? true : false;

const componentMustBeTyped: IsAny<typeof App> = false;
const props: InstanceType<typeof App>["$props"] = { count: 1 };

export type AppProps = InstanceType<typeof App>["$props"];
export type PublicInstance = InstanceType<typeof Public>;

void componentMustBeTyped;
void props;
