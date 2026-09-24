/** Compile-only assertions proving Mention item inference. */

import {
  MentionItem,
  MentionRoot,
  detectMention,
  type MentionInsertTransform,
  type MentionItemSlotState,
  type MentionLoadStatus,
  type MentionMatch,
  type MentionRootExpose,
  type MentionSlotState,
  type MentionTrigger,
} from "./mention.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface User {
  readonly id: number;
  readonly login: string;
}

declare const users: readonly User[];

type _Status = Expect<Equal<MentionLoadStatus, "error" | "idle" | "loading" | "success">>;
type _Detect = Expect<Equal<ReturnType<typeof detectMention>, MentionMatch | null>>;
type _Insert = Expect<
  Equal<MentionInsertTransform<User>, (item: User, trigger: MentionTrigger, text: string) => string>
>;

// `items` decides T for the select emit, the insert transform, and itemText.
MentionRoot({
  items: users,
  itemText: (user) => {
    type _ItemText = Expect<Equal<typeof user, User>>;
    return user.login;
  },
  insert: (user, trigger) => {
    type _InsertItem = Expect<Equal<typeof user, User>>;
    return `${trigger.char}${user.login} `;
  },
  onSelect: (user, trigger) => {
    type _Selected = Expect<Equal<typeof user, User>>;
    type _Trigger = Expect<Equal<typeof trigger, MentionTrigger>>;
  },
  "onQuery-change": (query, trigger) => {
    type _Query = Expect<Equal<typeof query, string | null>>;
    type _QueryTrigger = Expect<Equal<typeof trigger, MentionTrigger | null>>;
  },
});

// `loadItems` alone decides T.
MentionRoot({
  loadItems: async (query, trigger, { signal }) => {
    type _Signal = Expect<Equal<typeof signal, AbortSignal>>;
    void query;
    void trigger;
    return users;
  },
  onSelect: (user) => {
    type _Loaded = Expect<Equal<typeof user, User>>;
  },
  filter: false,
});

MentionRoot({
  items: ["bug", "docs"],
  triggers: [
    { char: "#", pattern: /^[\w-]*$/u },
    { char: ":", minChars: 2, allowSpaces: false },
  ],
  "onUpdate:modelValue": (text) => {
    type _Text = Expect<Equal<typeof text, string>>;
  },
});

// @ts-expect-error insert transforms receive the item type.
MentionRoot({ items: users, insert: (user: string) => user });

// @ts-expect-error filter functions receive the item type.
MentionRoot({ items: users, filter: (user: number) => user > 0 });

// @ts-expect-error triggers require a char.
MentionRoot({ items: users, triggers: [{ minChars: 1 }] });

type RootContext = NonNullable<ReturnType<typeof MentionRoot<User>>["__ctx"]>;
type _RootSlots = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], MentionSlotState<User>>
>;

type ItemContext = NonNullable<ReturnType<typeof MentionItem<User>>["__ctx"]>;
type _ItemValue = Expect<Equal<ItemContext["props"]["value"], User>>;
type _ItemSlots = Expect<
  Equal<Parameters<NonNullable<ItemContext["slots"]["default"]>>[0], MentionItemSlotState<User>>
>;

declare const exposed: MentionRootExpose<User>;
type _ExposeSelect = Expect<Equal<Parameters<typeof exposed.select>[0], User>>;

// @ts-expect-error MentionItem requires a value.
MentionItem({});
