import { parseSfc } from "@vizejs/native";
import { MappedModule } from "@vizejs/builder-shared/mapped-module";
import sources from "webpack-sources";
import type { LoaderContext } from "./types.ts";

export default function vizeStyleLoader(
  this: LoaderContext<Record<string, never>>,
  source: string,
): void {
  const callback = this.async();
  try {
    const query = new URLSearchParams(this.resourceQuery.slice(1));
    const text = query.get("index");
    if (text === null || !/^(0|[1-9]\d*)$/.test(text))
      throw new Error("[vize] Invalid style block index.");
    const descriptor = parseSfc(source, { filename: this.resourcePath });
    const style = descriptor.styles[Number(text)];
    // A removed block can briefly remain in the previous development graph.
    if (!style) {
      if (!query.has("dev"))
        throw new Error(`[vize] Style block ${text} is missing in ${this.resourcePath}.`);
      callback(null, "");
      return;
    }
    const bytes = Buffer.from(source);
    const start = bytes.subarray(0, style.loc.start).toString("utf8").length;
    const end = bytes.subarray(0, style.loc.end).toString("utf8").length;
    if (source.slice(start, end) !== style.content)
      throw new Error("[vize] Invalid native style geometry.");
    const mapped = new MappedModule(
      source,
      new sources.OriginalSource(source, this.resourcePath).map({ columns: true }),
    );
    mapped.replace(end, source.length, "");
    mapped.replace(0, start, "");
    callback(null, mapped.code, mapped.map);
  } catch (error) {
    callback(error instanceof Error ? error : new Error(String(error)));
  }
}
