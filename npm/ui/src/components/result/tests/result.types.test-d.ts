import { Result } from "../index.ts";
import type { ResultProps, ResultStatus } from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _StatusStaysClosed = Expect<
  Equal<ResultStatus, "success" | "error" | "info" | "warning" | "not-found" | "empty">
>;
type _PublicPropsFeedTheSfc = Expect<
  ResultProps extends InstanceType<typeof Result>["$props"] ? true : false
>;

export const props = {
  status: "not-found",
  title: "Page not found",
} satisfies ResultProps;

// @ts-expect-error status is a closed public contract.
export const badStatus: ResultProps = { title: "Missing", status: "404" };
