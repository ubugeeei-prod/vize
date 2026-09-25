/** Compile-only assertions for the public Pager contract. */

import { Pager, PagerPage, PagerTab, type PagerChangeReason, type PagerExpose } from "./pager.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type PagerProps<PageId extends string> = Parameters<typeof Pager<PageId>>[0];

/** Infers page ids exactly as a template usage would. */
declare function inferPage<PageId extends string>(props: PagerProps<PageId>): PageId;

declare const pager: PagerExpose<"feed" | "saved">;
const inferred = inferPage({ pages: ["feed", "saved"] });

type _InfersUnion = Expect<Equal<typeof inferred, "feed" | "saved">>;
type _ActiveIsTyped = Expect<Equal<typeof pager.active, "feed" | "saved">>;
type _ReasonIsClosed = Expect<Equal<PagerChangeReason, "api" | "keyboard" | "scroll" | "tab">>;

const props: PagerProps<"feed" | "saved"> = {
  pages: ["feed", "saved"],
  onChange: (page: "feed" | "saved", previous: "feed" | "saved", reason: PagerChangeReason) => {
    void page;
    void previous;
    void reason;
  },
};
const tabProps: InstanceType<typeof PagerTab>["$props"] = { page: "feed" };
const pageProps: InstanceType<typeof PagerPage>["$props"] = { page: "saved" };

pager.goTo("saved");

// @ts-expect-error unknown pages are rejected.
pager.goTo("other");

void pageProps;
void props;
void tabProps;
