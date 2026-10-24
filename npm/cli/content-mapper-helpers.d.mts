/** Shared type helpers imported by Vize Content Mapper virtual modules. */
type VizeIsAny<T> = 0 extends 1 & T ? true : false;

export type VizeComponentAttrCamel<S extends string> = S extends `${infer Head}-${infer Tail}`
  ? `${Head}${Capitalize<VizeComponentAttrCamel<Tail>>}`
  : S;

// @ts-ignore TS2694/TS2307: older or temporarily unavailable Vue types degrade to unchecked attrs.
type VizeNativeElements = import("vue").NativeElements;

type VizeNativeElementNames = Extract<keyof VizeNativeElements, string>;

type VizeComponentNativeAttrNames = {
  [K in VizeNativeElementNames]: Extract<keyof VizeNativeElements[K], string>;
}[VizeNativeElementNames];

export type VizeComponentGlobalHtmlAttrs =
  VizeIsAny<VizeNativeElements> extends true
    ? {}
    : {
        [K in VizeComponentNativeAttrNames as K | VizeComponentAttrCamel<K>]?: unknown;
      } & {
        [K in `data-${string}` | `data${Capitalize<string>}`]?: unknown;
      };
