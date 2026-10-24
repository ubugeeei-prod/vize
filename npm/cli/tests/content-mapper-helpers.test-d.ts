import type {
  VizeComponentAttrCamel,
  VizeComponentGlobalHtmlAttrs,
} from "vize/content-mapper-helpers";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Value extends true> = Value;

export type CamelizesNativeAttributeNames = Expect<
  Equal<VizeComponentAttrCamel<"accept-charset">, "acceptCharset">
>;
export type AcceptsHyphenatedDataAttributes = Expect<
  "data-test-id" extends keyof VizeComponentGlobalHtmlAttrs ? true : false
>;
export type AcceptsCamelizedDataAttributes = Expect<
  "dataTestId" extends keyof VizeComponentGlobalHtmlAttrs ? true : false
>;
