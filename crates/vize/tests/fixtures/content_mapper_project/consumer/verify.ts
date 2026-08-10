import CallSignatureChild from "./CallSignatureChild.vue";
import DefaultModelChild from "./DefaultModelChild.vue";
import GenericChild from "./GenericChild.vue";
import ModelChild from "./ModelChild.vue";
import Options from "./Options.vue";
import RuntimeChild from "./RuntimeChild.vue";
import type { AppProps, PublicInstance } from "./main";
import { readChildCount } from "./javascript-consumer";
import { renderChild } from "./jsx-consumer";

type IsAny<T> = 0 extends 1 & T ? true : false;
type CallSignatureChildInstance = InstanceType<typeof CallSignatureChild>;
type DefaultModelChildInstance = InstanceType<typeof DefaultModelChild>;
type GenericChildInstance = InstanceType<typeof GenericChild>;
type ModelChildInstance = InstanceType<typeof ModelChild>;
type OptionsInstance = InstanceType<typeof Options>;
type RuntimeChildInstance = InstanceType<typeof RuntimeChild>;

const propsMustBeTyped: IsAny<AppProps> = false;
const publicInstanceMustBeTyped: IsAny<PublicInstance> = false;
const publicSlotMustBeTyped: IsAny<PublicInstance["$slots"]["default"]> = false;
const publicExposeMustBeTyped: IsAny<PublicInstance["focus"]> = false;
const javascriptReturnMustBeTyped: IsAny<ReturnType<typeof readChildCount>> = false;
const jsxReturnMustBeTyped: IsAny<ReturnType<typeof renderChild>> = false;
const valid: AppProps = { count: 1 };
// @ts-expect-error count must remain a number through declaration emit
const invalid: AppProps = { count: "wrong" };
const optionsCountMustBeTyped: IsAny<OptionsInstance["count"]> = false;
const optionsLabelMustBeTyped: IsAny<OptionsInstance["label"]> = false;
const optionsComputedMustBeTyped: IsAny<OptionsInstance["doubled"]> = false;
const optionsMethodMustBeTyped: IsAny<OptionsInstance["increment"]> = false;
declare const options: OptionsInstance;
declare const callSignatureChild: CallSignatureChildInstance;
declare const defaultModelChild: DefaultModelChildInstance;
declare const genericChild: GenericChildInstance;
declare const modelChild: ModelChildInstance;
declare const runtimeChild: RuntimeChildInstance;
callSignatureChild.$emit("submit", true);
// @ts-expect-error submit payload must remain boolean through declaration emit
callSignatureChild.$emit("submit", "yes");
defaultModelChild.$emit("update:modelValue", 1);
// @ts-expect-error default model update payload must remain number through declaration emit
defaultModelChild.$emit("update:modelValue", "1");
genericChild.$emit("pick", "value");
// @ts-expect-error generic event constraint must remain string through declaration emit
genericChild.$emit("pick", 1);
modelChild.$emit("update:title", "value");
// @ts-expect-error model update payload must remain string through declaration emit
modelChild.$emit("update:title", 1);
runtimeChild.$emit("cancel", "reason");
// @ts-expect-error cancel payload must remain string through declaration emit
runtimeChild.$emit("cancel", false);
const optionsCount: number = options.count;
const optionsLabel: string = options.label;
const optionsComputed: number = options.doubled;
const optionsMethodResult: number = options.increment(1);
// @ts-expect-error the authored method parameter must remain a number
options.increment("1");

declare const publicComponent: PublicInstance;
const publicValue: string = publicComponent.$props.value;
const publicModel: number | undefined = publicComponent.$props.modelValue;
const javascriptResult: number = readChildCount({ count: 1 });
// @ts-expect-error JavaScript declarations must preserve the Vue prop type
readChildCount({ count: "wrong" });
publicComponent.$emit("select", "value");
publicComponent.$emit("update:modelValue", 1);
publicComponent.$slots.default?.({ value: "slot" });
// @ts-expect-error default slot value must remain a string
publicComponent.$slots.default?.({ value: 1 });
publicComponent.focus();

void propsMustBeTyped;
void publicInstanceMustBeTyped;
void publicSlotMustBeTyped;
void publicExposeMustBeTyped;
void javascriptReturnMustBeTyped;
void jsxReturnMustBeTyped;
void valid;
void invalid;
void optionsCountMustBeTyped;
void optionsLabelMustBeTyped;
void optionsComputedMustBeTyped;
void optionsMethodMustBeTyped;
void optionsCount;
void optionsLabel;
void optionsComputed;
void optionsMethodResult;
void publicValue;
void publicModel;
void javascriptResult;
