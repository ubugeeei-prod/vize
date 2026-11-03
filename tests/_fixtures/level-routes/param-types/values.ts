import appRouter from "./router";

declare const dynamicId: string;

// Accepted: one of every value kind a param takes.
appRouter.push({ name: "item", params: { id: "a" } });
appRouter.push({ name: "item", params: { id: 7 } });
appRouter.push({ name: "item", params: { id: -7 } });
appRouter.push({ name: "item", params: { id: `item-${dynamicId}` } });
appRouter.push({ name: "item", params: { id: dynamicId } });
appRouter.push({ name: "files", params: { segments: ["a", "b"] } });
appRouter.push({ name: "files", params: { segments: "a" } });
appRouter.push({ name: "docs", params: { rest: [] } });
appRouter.push({ name: "find", params: { q: null } });
appRouter.push({ name: "find", params: { q: undefined } });
appRouter.push({ name: "find", params: { q: "" } });
appRouter.push({ name: "find" });

// Rejected: each value kind a param cannot take.
appRouter.push({ name: "item", params: { id: ["a", "b"] } });
appRouter.push({ name: "item", params: { id: "" } });
appRouter.push({ name: "item", params: { id: `` } });
appRouter.push({ name: "item", params: { id: null } });
appRouter.push({ name: "item", params: { id: undefined } });
appRouter.push({ name: "item", params: { id: void 0 } });
appRouter.push({ name: "item", params: { id: true } });
appRouter.push({ name: "item", params: { id: { value: 1 } } });
appRouter.push({ name: "item", params: { id: () => "a" } });
appRouter.push({ name: "item", params: { id: 1n } });
appRouter.push({ name: "item", params: { id: /a/ } });
appRouter.push({ name: "files", params: { segments: [] } });
appRouter.push({ name: "find", params: { q: ["a"] } });
appRouter.push({ name: "find", params: { q: false } });

// Outside the domain: not a named location with static params.
appRouter.push({ name: "item", params: dynamicParams() });
appRouter.push({ name: "item", ...extra() });
appRouter.push({ name: "item", params: { ...extra() } });
appRouter.push({ path: "/items/1", name: "nope" });
appRouter.push("/items/1");

// A parameter named `undefined` is a string here, not the undefined value.
function undefinedShadow(undefined: string) {
  appRouter.push({ name: "item", params: { id: undefined } });
}
undefinedShadow(dynamicId);

function dynamicParams() {
  return { id: "a" };
}

function extra() {
  return {};
}
