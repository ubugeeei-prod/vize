import Options from "../types/Options.vue";
import Data from "../types/Data.vue";
import Computed from "../types/Computed.vue";
import Methods from "../types/Methods.vue";
import PropsEmits from "../types/PropsEmits.vue";
import SetupReturn from "../types/SetupReturn.vue";
import Inherited from "../types/Inherited.vue";

type IsAny<T> = 0 extends 1 & T ? true : false;

type OptionsInstance = InstanceType<typeof Options>;
const optionsInstanceIsAny: false = null as unknown as IsAny<OptionsInstance>;
declare const options: OptionsInstance;
const optionsCount: number = options.count;
const optionsCountIsAny: false = null as unknown as IsAny<OptionsInstance["count"]>;
// @ts-expect-error data member keeps its exact type
const optionsCountAsString: string = options.count;
// @ts-expect-error undeclared members stay absent
const optionsMissingMember: unknown = options.missingMember;

declare const data: InstanceType<typeof Data>;
const dataCount: number = data.count;
const dataLabel: string = data.label;
const dataFlag: boolean = data.flag;
const dataDepth: number = data.nested.inner.depth;
const dataList: number[] = data.list;
const dataPair: [string, number] = data.pair;
const dataWidened: string = data.widened;
const dataLiteral: "exact" = data.literal;
const dataMaybe: string | null = data.maybe;
const dataLabelIsAny: false = null as unknown as IsAny<typeof data.label>;
// @ts-expect-error a nested data member keeps its exact type
const dataDepthAsString: string = data.nested.inner.depth;
// @ts-expect-error an array data member is not widened to any
const dataListAsStrings: string[] = data.list;
// @ts-expect-error a const-asserted literal is not widened
const dataLiteralWidened: "other" = data.literal;
// @ts-expect-error a nullable data member keeps its null branch
const dataMaybeNonNull: string = data.maybe;

declare const computed: InstanceType<typeof Computed>;
const computedDoubled: number = computed.doubled;
const computedQuadrupled: number = computed.quadrupled;
const computedLabel: string = computed.label;
computed.label = "9";
const computedDoubledIsAny: false = null as unknown as IsAny<typeof computed.doubled>;
// @ts-expect-error a computed getter keeps its exact type
const computedDoubledAsString: string = computed.doubled;
// @ts-expect-error a writable computed keeps its setter type
computed.label = 9;

declare const methods: InstanceType<typeof Methods>;
const methodsAdd: number = methods.add(1);
const methodsDescribe: string = methods.describe("x");
const methodsDescribeNumber: string = methods.describe(2);
const methodsLoad: Promise<{ id: string }> = methods.load("id");
const methodsWalk: Generator<number, void, unknown> = methods.walk();
const methodsAddIsAny: false = null as unknown as IsAny<typeof methods.add>;
// @ts-expect-error a method parameter keeps its exact type
methods.add("1");
// @ts-expect-error a union parameter rejects an outside member
methods.describe(true);
// @ts-expect-error an async method keeps its awaited type
const methodsLoadSync: { id: string } = methods.load("id");

declare const propsEmits: InstanceType<typeof PropsEmits>;
const propsTitle: string = propsEmits.title;
const propsSize: number = propsEmits.size;
const propsItems: string[] = propsEmits.items;
const propsHeading: string = propsEmits.heading;
propsEmits.emitPick();
propsEmits.$emit("pick", "ok");
const propsTitleIsAny: false = null as unknown as IsAny<typeof propsEmits.title>;
// @ts-expect-error a runtime prop keeps its declared type
const propsTitleAsNumber: number = propsEmits.title;
// @ts-expect-error a PropType array prop keeps its element type
const propsItemsAsNumbers: number[] = propsEmits.items;
// @ts-expect-error an emit payload stays exact
propsEmits.$emit("pick", 1);

declare const setupReturn: InstanceType<typeof SetupReturn>;
const setupTotal: number = setupReturn.total;
const setupDoubled: number = setupReturn.doubledTotal;
const setupSeed: string = setupReturn.seed;
setupReturn.bump(1);
const setupTotalIsAny: false = null as unknown as IsAny<typeof setupReturn.total>;
// @ts-expect-error a setup ref is unwrapped on the public instance
const setupTotalRef: { value: number } = setupReturn.total;
// @ts-expect-error a setup computed keeps its exact type
const setupDoubledAsString: string = setupReturn.doubledTotal;
// @ts-expect-error a method keeps its parameter type
setupReturn.bump("1");

declare const inherited: InstanceType<typeof Inherited>;
const inheritedOwn: number = inherited.own;
const inheritedCount: number = inherited.inheritedCount;
const inheritedGreet: string = inherited.greet();
const inheritedCountIsAny: false = null as unknown as IsAny<typeof inherited.inheritedCount>;
// @ts-expect-error an extends-inherited member keeps its type
const inheritedCountAsString: string = inherited.inheritedCount;
// @ts-expect-error a mixin-inherited member keeps its type
const inheritedGreetAsNumber: number = inherited.greet();

const optionsElement = <Options />;
const propsElement = <PropsEmits title="ok" items={["a"]} />;

void [
  optionsInstanceIsAny,
  optionsCount,
  optionsCountIsAny,
  optionsCountAsString,
  optionsMissingMember,
  dataCount,
  dataLabel,
  dataFlag,
  dataDepth,
  dataList,
  dataPair,
  dataWidened,
  dataLiteral,
  dataMaybe,
  dataLabelIsAny,
  dataDepthAsString,
  dataListAsStrings,
  dataLiteralWidened,
  dataMaybeNonNull,
  computedDoubled,
  computedQuadrupled,
  computedLabel,
  computedDoubledIsAny,
  computedDoubledAsString,
  methodsAdd,
  methodsDescribe,
  methodsDescribeNumber,
  methodsLoad,
  methodsWalk,
  methodsAddIsAny,
  methodsLoadSync,
  propsTitle,
  propsSize,
  propsItems,
  propsHeading,
  propsTitleIsAny,
  propsTitleAsNumber,
  propsItemsAsNumbers,
  setupTotal,
  setupDoubled,
  setupSeed,
  setupTotalIsAny,
  setupTotalRef,
  setupDoubledAsString,
  inheritedOwn,
  inheritedCount,
  inheritedGreet,
  inheritedCountIsAny,
  inheritedCountAsString,
  inheritedGreetAsNumber,
  optionsElement,
  propsElement,
];
