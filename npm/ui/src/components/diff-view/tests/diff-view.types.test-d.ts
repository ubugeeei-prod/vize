import { DiffView } from "../index.ts";
import type { DiffLineKind, DiffViewProps } from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _KindsStayClosed = Expect<Equal<DiffLineKind, "context" | "add" | "remove" | "meta">>;
type _PublicPropsFeedTheSfc = Expect<
  DiffViewProps extends InstanceType<typeof DiffView>["$props"] ? true : false
>;

export const props = {
  label: "readme.md",
  lines: [{ kind: "add", text: "new" }],
} satisfies DiffViewProps;

// @ts-expect-error line kinds are a closed public contract.
export const badLine: DiffViewProps = { lines: [{ kind: "edit", text: "new" }] };
