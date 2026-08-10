import "vue";
import Public from "../types/Public.vue";
import Callable from "../types/Callable.vue";
import RuntimeObject from "../types/RuntimeObject.vue";
import RuntimeArray from "../types/RuntimeArray.vue";
import Generic from "../types/Generic.vue";
import Union from "../types/Union.vue";
import NativeListenerProp from "../types/NativeListenerProp.vue";
import NoEmits from "../types/NoEmits.vue";

declare module "vue" {
  interface ComponentCustomProperties {
    $service: { ping(value: string): number };
  }
}

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
const serviceResult: number = instance.$service.ping("ok");
const attrs: Record<string, unknown> = instance.$attrs;
const refs: Record<string, unknown> = instance.$refs;
instance.$emit("select", "ok");
instance.$emit("update:modelValue", 1);
instance.$slots.default?.({ value: "ok" });
instance.close(true);
type PublicEmit = PublicInstance["$emit"];
type PublicEmitParameters = Parameters<PublicEmit>;
type PublicEmitWithoutThis = OmitThisParameter<PublicEmit>;
type PublicEmitThis = ThisParameterType<PublicEmit>;
declare const publicEmitWithoutThis: PublicEmitWithoutThis;
const publicEmitThisIsUnknown: true = null as unknown as __VizeIsUnknown<PublicEmitThis>;

// @ts-expect-error emitted required props stay required
const missingProps: PublicInstance["$props"] = {};
// @ts-expect-error emitted record payload stays exact
instance.$emit("select", 1);
// @ts-expect-error emitted model payload stays exact
instance.$emit("update:modelValue", "1");
// @ts-expect-error emitted event names stay exact
instance.$emit("other", 1);
// @ts-expect-error emitted slots stay exact
instance.$slots.default?.({ value: 1 });
// @ts-expect-error emitted expose stays exact
instance.close("yes");
// @ts-expect-error emitted Parameters cannot recover the compatibility event
const broadEmitParameters: PublicEmitParameters = ["other", 1];
// @ts-expect-error emitted OmitThisParameter cannot recover the compatibility event
publicEmitWithoutThis("other", 1);
// @ts-expect-error emitted Function.call keeps the exact event surface
instance.$emit.call(instance, "other", 1);
// @ts-expect-error emitted Function.bind keeps the exact event surface
instance.$emit.bind(instance)("other", 1);

type NoEmitsInstance = InstanceType<typeof NoEmits>;
declare const broadInstance: import("vue").ComponentPublicInstance;
const noEmits = broadInstance as NoEmitsInstance;
noEmits.ping();
type NoEmitsParameters = Parameters<NoEmitsInstance["$emit"]>;
type NoEmitsWithoutThis = OmitThisParameter<NoEmitsInstance["$emit"]>;
declare const noEmitsWithoutThis: NoEmitsWithoutThis;
// @ts-expect-error emitted empty event surface rejects calls
noEmits.$emit("other", 1);
// @ts-expect-error emitted Parameters cannot broaden an empty event surface
const noEmitsBroadParameters: NoEmitsParameters = ["other", 1];
// @ts-expect-error emitted OmitThisParameter cannot broaden an empty event surface
noEmitsWithoutThis("other", 1);
// @ts-expect-error emitted Function.call cannot broaden an empty event surface
noEmits.$emit.call(noEmits, "other", 1);
// @ts-expect-error emitted Function.bind cannot broaden an empty event surface
noEmits.$emit.bind(noEmits)("other", 1);

declare const callable: InstanceType<typeof Callable>;
callable.$emit("commit", "ok");
// @ts-expect-error emitted callable payload stays exact
callable.$emit("commit", 1);
// @ts-expect-error emitted callable event stays exact
callable.$emit("other");

declare const runtimeObject: InstanceType<typeof RuntimeObject>;
runtimeObject.$emit("save", "ok");
// @ts-expect-error emitted runtime object payload stays exact
runtimeObject.$emit("save", 1);
// @ts-expect-error emitted runtime object event stays exact
runtimeObject.$emit("other");

declare const runtimeArray: InstanceType<typeof RuntimeArray>;
runtimeArray.$emit("open");
// @ts-expect-error emitted runtime array event names stay exact
runtimeArray.$emit("other");

declare const generic: InstanceType<typeof Generic>;
const genericItem: string = generic.$props.item;
generic.$emit("pick", "ok");
// @ts-expect-error emitted generic payload stays exact
generic.$emit("pick", 1);

const camel = <Public modelValue={1} someValue="ok" />;
const kebab = <Public model-value={1} some-value="ok" />;
const mixed = <Public modelValue={1} some-value="ok" optional-value />;
const fallthrough = (
  <Public
    modelValue={1}
    someValue="ok"
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
const unionText = <Union kind="text" textValue="ok" />;
const unionCount = <Union kind="count" count-value={1} />;
const nativeListenerProp = <NativeListenerProp onClick={(value) => value.toUpperCase()} />;
// @ts-expect-error emitted input keeps required aliases
const missing = <Public />;
// @ts-expect-error emitted input rejects excess props
const extra = <Public modelValue={1} someValue="ok" unrelated />;
// @ts-expect-error emitted input keeps listener payloads exact
const wrongListener = <Public modelValue={1} someValue="ok" onSelect={(value: number) => value} />;
// @ts-expect-error emitted input rejects undeclared listeners
const unknownListener = <Public modelValue={1} someValue="ok" onOther={() => undefined} />;
// @ts-expect-error emitted generic input rejects excess props
const extraGeneric = <Generic item="value" unrelated />;
// @ts-expect-error emitted union branches keep their own required props
const missingUnionBranchProp = <Union kind="text" />;
// @ts-expect-error emitted union branches cannot be mixed
const mixedUnionBranches = <Union kind="text" textValue="ok" countValue={1} />;
// @ts-expect-error emitted onClick component props stay required
const missingNativeListenerProp = <NativeListenerProp />;
const wrongNativeListenerProp = (
  // @ts-expect-error emitted onClick component props win over native fallthrough
  <NativeListenerProp onClick={(value: MouseEvent) => value.preventDefault()} />
);

void unknownUsesPublicInstance;
void anyIsAuthoredInput;
void neverIsAuthoredInput;
void objectIsAuthoredInput;
void anyInstanceInferenceUsesPublicInstance;
void instanceIsAny;
void requiredModel;
void serviceResult;
void attrs;
void refs;
void missingProps;
void genericItem;
void publicEmitThisIsUnknown;
void broadEmitParameters;
void noEmitsBroadParameters;
void [
  camel,
  kebab,
  mixed,
  fallthrough,
  unionText,
  unionCount,
  nativeListenerProp,
  missing,
  extra,
  wrongListener,
  unknownListener,
  extraGeneric,
  missingUnionBranchProp,
  mixedUnionBranches,
  missingNativeListenerProp,
  wrongNativeListenerProp,
];
