import Public from "./Public.vue";
import type { ComponentPublicInstance } from "vue";
import Callable from "./Callable.vue";
import RuntimeObject from "./RuntimeObject.vue";
import RuntimeArray from "./RuntimeArray.vue";
import Generic from "./Generic.vue";
import Union from "./Union.vue";
import NativeListenerProp from "./NativeListenerProp.vue";
import NoEmits from "./NoEmits.vue";

type IsAny<T> = 0 extends 1 & T ? true : false;
const unknownUsesPublicInstance: true = null as unknown as __VizeIsUnknown<unknown>;
const anyIsAuthoredInput: false = null as unknown as __VizeIsUnknown<any>;
const neverIsAuthoredInput: false = null as unknown as __VizeIsUnknown<never>;
const objectIsAuthoredInput: false = null as unknown as __VizeIsUnknown<{}>;
const anyInstanceInferenceUsesPublicInstance: true =
  null as unknown as __VizeUsePublicInstance<any>;
type PublicInstance = InstanceType<typeof Public>;
declare const instance: PublicInstance;

const instanceIsAny: false = null as unknown as IsAny<PublicInstance>;
const requiredModel: number = instance.$props.modelValue;
const defaultedModel: string | undefined = instance.$props.title;
const optionalModel: boolean | undefined = instance.$props.enabled;
const serviceResult: number = instance.$service.ping("ok");
const attrs: Record<string, unknown> = instance.$attrs;
const refs: Record<string, unknown> = instance.$refs;

instance.$emit("select", "ok");
instance.$emit("clear");
instance.$emit("update:modelValue", 1);
instance.$emit("update:title", "next");
instance.$emit("update:enabled", undefined);
instance.$slots.default?.({ value: "ok" });
instance.close(true);
type PublicEmit = PublicInstance["$emit"];
type PublicEmitParameters = Parameters<PublicEmit>;
type PublicEmitWithoutThis = OmitThisParameter<PublicEmit>;
type PublicEmitThis = ThisParameterType<PublicEmit>;
declare const publicEmitWithoutThis: PublicEmitWithoutThis;
const publicEmitThisIsUnknown: true = null as unknown as __VizeIsUnknown<PublicEmitThis>;

// @ts-expect-error record emit payload stays exact
instance.$emit("select", 1);
// @ts-expect-error model emit payload stays exact
instance.$emit("update:modelValue", "1");
// @ts-expect-error event names stay exact
instance.$emit("other", 1);
// @ts-expect-error slot payload stays exact
instance.$slots.default?.({ value: 1 });
// @ts-expect-error exposed members stay exact
instance.close("yes");
// @ts-expect-error Parameters cannot recover the compatibility event
const broadEmitParameters: PublicEmitParameters = ["other", 1];
// @ts-expect-error OmitThisParameter cannot recover the compatibility event
publicEmitWithoutThis("other", 1);
// @ts-expect-error Function.call keeps the exact event surface
instance.$emit.call(instance, "other", 1);
// @ts-expect-error Function.bind keeps the exact event surface
instance.$emit.bind(instance)("other", 1);

type NoEmitsInstance = InstanceType<typeof NoEmits>;
declare const broadInstance: ComponentPublicInstance;
const noEmits = broadInstance as NoEmitsInstance;
noEmits.ping();
type NoEmitsParameters = Parameters<NoEmitsInstance["$emit"]>;
type NoEmitsWithoutThis = OmitThisParameter<NoEmitsInstance["$emit"]>;
declare const noEmitsWithoutThis: NoEmitsWithoutThis;
// @ts-expect-error an SFC without emits has no public event names
noEmits.$emit("other", 1);
// @ts-expect-error an empty event surface cannot broaden through Parameters
const noEmitsBroadParameters: NoEmitsParameters = ["other", 1];
// @ts-expect-error an empty event surface cannot broaden through OmitThisParameter
noEmitsWithoutThis("other", 1);
// @ts-expect-error an empty event surface cannot broaden through Function.call
noEmits.$emit.call(noEmits, "other", 1);
// @ts-expect-error an empty event surface cannot broaden through Function.bind
noEmits.$emit.bind(noEmits)("other", 1);

type CallableInstance = InstanceType<typeof Callable>;
declare const callable: CallableInstance;
callable.$emit("commit", "ok");
callable.$emit("cancel");
// @ts-expect-error callable overload payload stays exact
callable.$emit("commit", 1);
// @ts-expect-error callable overload event stays exact
callable.$emit("other");

type RuntimeObjectInstance = InstanceType<typeof RuntimeObject>;
declare const runtimeObject: RuntimeObjectInstance;
runtimeObject.$emit("save", "ok");
runtimeObject.$emit("reset");
// @ts-expect-error runtime object payload stays exact
runtimeObject.$emit("save", 1);
// @ts-expect-error runtime object event stays exact
runtimeObject.$emit("other");

type RuntimeArrayInstance = InstanceType<typeof RuntimeArray>;
declare const runtimeArray: RuntimeArrayInstance;
runtimeArray.$emit("open");
runtimeArray.$emit("close", 1);
// @ts-expect-error runtime array event names stay exact
runtimeArray.$emit("other");

type GenericInstance = InstanceType<typeof Generic>;
declare const generic: GenericInstance;
const genericItem: string = generic.$props.item;
generic.$emit("pick", "ok");
// @ts-expect-error generic fallback payload stays exact
generic.$emit("pick", 1);

const camel = <Public modelValue={1} someValue="ok" />;
const kebab = <Public model-value={1} some-value="ok" />;
const mixed = <Public modelValue={1} some-value="ok" optional-value />;
const optionalModels = <Public modelValue={1} someValue="ok" title="next" enabled />;
const listeners = (
  <Public
    modelValue={1}
    someValue="ok"
    onSelect={(value) => {
      const notAny: false = null as unknown as IsAny<typeof value>;
      void notAny;
      return value.toUpperCase();
    }}
  />
);
const modelListener = (
  <Public
    modelValue={1}
    someValue="ok"
    {...{ "onUpdate:modelValue": (value: number) => value.toFixed() }}
  />
);
const fallthrough = (
  <Public
    modelValue={1}
    someValue="ok"
    class="public"
    style="color:red"
    data-testid="public"
    aria-label="Public"
    onClick={(event) => {
      const mouse: MouseEvent = event;
      const notAny: false = null as unknown as IsAny<typeof event>;
      void notAny;
      return mouse.preventDefault();
    }}
    onInput={(event) => {
      const input: InputEvent = event;
      return input.preventDefault();
    }}
  />
);
const callableListeners = (
  <Callable onCommit={(value) => value.toUpperCase()} onCancel={() => undefined} />
);
const runtimeObjectListeners = (
  <RuntimeObject onSave={(value) => value.toUpperCase()} onReset={() => undefined} />
);
const runtimeArrayListeners = <RuntimeArray onOpen={() => undefined} onClose={() => undefined} />;
const genericListener = (
  <Generic
    item="value"
    onPick={(value) => {
      const exact: "value" = value;
      return exact.toUpperCase();
    }}
  />
);
const unionText = <Union kind="text" textValue="ok" />;
const unionCount = <Union kind="count" count-value={1} />;
const nativeListenerProp = <NativeListenerProp onClick={(value) => value.toUpperCase()} />;

// @ts-expect-error both spellings of required props are missing
const missing = <Public />;
// @ts-expect-error the required model cannot disappear
const missingModel = <Public someValue="ok" />;
// @ts-expect-error wrong prop values stay exact
const wrongModel = <Public model-value="1" some-value="ok" />;
// @ts-expect-error unrelated props cannot escape generic inference
const extra = <Public modelValue={1} someValue="ok" unrelated />;
// @ts-expect-error declared listener payload stays exact despite the DOM select event
const wrongListener = <Public modelValue={1} someValue="ok" onSelect={(value: number) => value} />;
// @ts-expect-error undeclared non-DOM listeners are not fallthrough attrs
const unknownListener = <Public modelValue={1} someValue="ok" onOther={() => undefined} />;
const wrongModelListener = (
  // @ts-expect-error model listener payload stays exact
  <Public modelValue={1} someValue="ok" {...{ "onUpdate:modelValue": (value: string) => value }} />
);
// @ts-expect-error callable listener payload stays exact
const wrongCallable = <Callable onCommit={(value: number) => value} />;
// @ts-expect-error runtime object listener payload stays exact
const wrongRuntimeObject = <RuntimeObject onSave={(value: number) => value} />;
// @ts-expect-error runtime array event names stay exact
const wrongRuntimeArray = <RuntimeArray onOther={() => undefined} />;
// @ts-expect-error generic props do not allow excess-property escape
const extraGeneric = <Generic item="value" unrelated />;
// @ts-expect-error union branches keep their own required props
const missingUnionBranchProp = <Union kind="text" />;
// @ts-expect-error props from another union branch cannot be mixed in
const mixedUnionBranches = <Union kind="text" textValue="ok" countValue={1} />;
// @ts-expect-error an onClick component prop remains required
const missingNativeListenerProp = <NativeListenerProp />;
const wrongNativeListenerProp = (
  // @ts-expect-error an onClick component prop wins over native fallthrough
  <NativeListenerProp onClick={(value: MouseEvent) => value.preventDefault()} />
);

void unknownUsesPublicInstance;
void anyIsAuthoredInput;
void neverIsAuthoredInput;
void objectIsAuthoredInput;
void anyInstanceInferenceUsesPublicInstance;
void instanceIsAny;
void requiredModel;
void defaultedModel;
void optionalModel;
void serviceResult;
void attrs;
void refs;
void genericItem;
void publicEmitThisIsUnknown;
void broadEmitParameters;
void noEmitsBroadParameters;
void [
  camel,
  kebab,
  mixed,
  optionalModels,
  listeners,
  modelListener,
  fallthrough,
  callableListeners,
  runtimeObjectListeners,
  runtimeArrayListeners,
  genericListener,
  unionText,
  unionCount,
  nativeListenerProp,
  missing,
  missingModel,
  wrongModel,
  extra,
  wrongListener,
  unknownListener,
  wrongModelListener,
  wrongCallable,
  wrongRuntimeObject,
  wrongRuntimeArray,
  extraGeneric,
  missingUnionBranchProp,
  mixedUnionBranches,
  missingNativeListenerProp,
  wrongNativeListenerProp,
];
