import { defineTasks, moonScript, noCacheTask, runTask } from "../task-helpers.ts";

/**
 * Code generation tasks and local CLI smoke commands.
 *
 * JSON Schema remains the source of truth for public config declarations, while
 * MoonBit performs the post-processing that encodes Vize-specific API details
 * which the schema generator cannot express on its own. The CLI aliases stay in
 * the same module because they are the fastest manual way to inspect generated
 * config and compiler output after the generator changes.
 */
export const generationAndCliTasks = defineTasks({
  "gen:schema": noCacheTask(
    "pnpm exec pkl eval -f json npm/vize/pkl/jsonschema/generate.pkl -o npm/vize/schemas/vize.config.schema.json",
  ),
  "gen:types": noCacheTask(
    `${runTask("gen:schema")} && pnpm exec json2ts -i npm/vize/schemas/vize.config.schema.json -o npm/vize/src/types/generated.ts && ${moonScript("postprocess_types")}`,
  ),
  gen: noCacheTask(runTask("gen:types")),
  cli: noCacheTask(
    'sh -c \'if [ "${usage_debug:-$1}" = "true" ] || [ "$1" = "--debug" ]; then cargo install --path crates/vize --force --locked --debug && echo "Installed vize CLI (debug build)"; else cargo install --path crates/vize --force --locked && echo "Installed vize CLI (release build)"; fi\' --',
  ),
  "cli:help": noCacheTask("vize --help"),
  "cli:example": noCacheTask("vize './**/*.vue' -o . -v"),
  "cli:example-json": noCacheTask("vize './**/*.vue' -o . -f json -v"),
  "cli:example-ssr": noCacheTask("vize './**/*.vue' -o . -f json --ssr -v"),
  "cli:example-stats": noCacheTask("vize './**/*.vue' -f stats -v"),
});
