import {
  parseSfc,
  type SfcParsedAttributeNapi,
  type SfcParseResultNapi,
} from "../../../../npm/native/index.js";

const result: SfcParseResultNapi = parseSfc("<script setup vapor></script>", {
  includeAttrs: true,
});
const marker: boolean =
  result.scriptSetup?.attrs?.some(
    ({ name, value }) => name === "vapor" && typeof value === "string",
  ) ?? false;
const offset: number = result.template?.loc.start ?? 0;
const lang: string | null = result.script?.lang ?? null;
const customType: string | undefined = result.customBlocks[0]?.type;
const attribute: SfcParsedAttributeNapi = { name: "vapor", value: "false" };
type IsAny<T> = 0 extends 1 & T ? true : false;
const strictReturn: IsAny<ReturnType<typeof parseSfc>> = false;
// @ts-expect-error parser flags are boolean, not string values
parseSfc("", { includeAttrs: "true" });
// @ts-expect-error parsed attribute values preserve strings, including empty strings
const invalid: SfcParsedAttributeNapi = { name: "vapor", value: false };
void [marker, offset, lang, attribute, invalid, strictReturn, customType];
