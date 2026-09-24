/** Compile-only assertions for the public TagsInput contract and its tag type inference. */

import {
  TagsInput,
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
  TagsInputItemText,
  TagsInputRoot,
  splitTagText,
  type TagsInputAddSource,
  type TagsInputBy,
  type TagsInputInvalidEvent,
  type TagsInputInvalidReason,
  type TagsInputItemSlotState,
  type TagsInputItemState,
  type TagsInputRemoveSource,
  type TagsInputRootExpose,
  type TagsInputSlotState,
  type TagsInputState,
} from "./tags-input.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type RootProps<T> = Parameters<typeof TagsInputRoot<T>>[0];
type ItemProps<T> = Parameters<typeof TagsInputItem<T>>[0];

interface Topic {
  readonly id: number;
  readonly label: string;
}

type _StateIsClosed = Expect<Equal<TagsInputState, "disabled" | "empty" | "filled" | "readonly">>;
type _ItemStateIsClosed = Expect<Equal<TagsInputItemState, "disabled" | "editing" | "idle">>;
type _ReasonIsClosed = Expect<
  Equal<TagsInputInvalidReason, "duplicate" | "invalid" | "max" | "parse">
>;
type _AddSourceIsClosed = Expect<
  Equal<TagsInputAddSource, "api" | "blur" | "delimiter" | "enter" | "paste">
>;
type _RemoveSourceIsClosed = Expect<
  Equal<TagsInputRemoveSource, "api" | "backspace" | "delete" | "delete-button">
>;
type _AliasIsRoot = Expect<Equal<typeof TagsInput, typeof TagsInputRoot>>;
type _ModelFollowsT = Expect<
  Equal<NonNullable<RootProps<number>["modelValue"]>, readonly number[]>
>;
type _ParserReturnsT = Expect<
  Equal<NonNullable<RootProps<Topic>["parseTag"]>, (text: string) => Topic | null>
>;
type _ByAcceptsKeysOrComparator = Expect<
  Equal<TagsInputBy<Topic>, "id" | "label" | ((left: Topic, right: Topic) => boolean)>
>;
type _StringByIsComparatorOnly = Expect<
  Equal<TagsInputBy<string>, (left: string, right: string) => boolean>
>;
type _ItemValueFollowsT = Expect<Equal<ItemProps<Topic>["value"], Topic>>;
type _SlotTagsAreReadonly = Expect<Equal<TagsInputSlotState<Topic>["tags"], readonly Topic[]>>;
type _ItemSlotValue = Expect<Equal<TagsInputItemSlotState<number>["value"], number>>;
type _ExposeTags = Expect<Equal<TagsInputRootExpose<Topic>["tags"], readonly Topic[]>>;
type _ExposeAddTag = Expect<Equal<TagsInputRootExpose<Topic>["addTag"], (tag: Topic) => boolean>>;
type _InvalidTag = Expect<Equal<TagsInputInvalidEvent<number>["tag"], number | null>>;

// Inference: the tag type flows from `parseTag` into v-model and every event.
TagsInputRoot({
  defaultValue: [1, 2],
  parseTag: (text) => {
    type _TextIsString = Expect<Equal<typeof text, string>>;
    return Number.isNaN(Number(text)) ? null : Number(text);
  },
  tagText: (tag) => {
    type _TagIsNumber = Expect<Equal<typeof tag, number>>;
    return String(tag);
  },
  validate: (tag, tags) => {
    type _ValidateTags = Expect<Equal<typeof tags, readonly number[]>>;
    return tag > 0 || "Must be positive";
  },
  "onUpdate:modelValue": (value) => {
    type _UpdateIsNumberList = Expect<Equal<typeof value, readonly number[]>>;
  },
  onAdd: (tag, index, source) => {
    type _AddTag = Expect<Equal<typeof tag, number>>;
    type _AddIndex = Expect<Equal<typeof index, number>>;
    type _AddSource = Expect<Equal<typeof source, TagsInputAddSource>>;
  },
  onInvalid: (event) => {
    type _InvalidEvent = Expect<Equal<typeof event, TagsInputInvalidEvent<number>>>;
  },
});

// Inference from object models and a property-name `by` policy.
TagsInputRoot({
  by: "id",
  modelValue: [{ id: 1, label: "Vue" }] satisfies readonly Topic[],
  parseTag: (text): Topic => ({ id: text.length, label: text }),
  onEdit: (tag, previous) => {
    type _EditTag = Expect<Equal<typeof tag, Topic>>;
    type _EditPrevious = Expect<Equal<typeof previous, Topic>>;
  },
});

TagsInputRoot({
  // @ts-expect-error `by` keys must exist on the tag type.
  by: "missing",
  defaultValue: [{ id: 1, label: "Vue" }],
});

TagsInputRoot({
  defaultValue: ["vue"],
  // @ts-expect-error the parser must produce the inferred tag type.
  parseTag: (text: string) => text.length,
});

TagsInputRoot({
  // @ts-expect-error delimiters are strings.
  delimiters: [1],
});

// @ts-expect-error items require an index.
TagsInputItem({ value: "vue" });

const deleteProps: InstanceType<typeof TagsInputItemDelete>["$props"] = { label: "Delete" };
const textProps: InstanceType<typeof TagsInputItemText>["$props"] = {};
const inputProps: InstanceType<typeof TagsInputInput>["$props"] = {
  autocomplete: "off",
  placeholder: "Add",
};
// @ts-expect-error the input exposes no value prop; text is owned by the root.
const badInputProps: InstanceType<typeof TagsInputInput>["$props"] = { value: "x" };
void deleteProps;
void textProps;
void inputProps;
void badInputProps;
splitTagText("a,b", [","]);

// @ts-expect-error state tokens are a closed union.
const invalidState: TagsInputState = "busy";
void invalidState;
