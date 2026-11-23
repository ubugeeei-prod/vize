import { defineTasks, nodeTool, noCacheTask, runTask, steps } from "../task-helpers.ts";

export const generationAndCliTasks = defineTasks({
  "gen:schema": noCacheTask(
    "pnpm --dir npm/cli exec pkl eval --project-dir pkl -f json pkl/jsonschema/generate.pkl -o schemas/vize.config.schema.json",
  ),
  "gen:types": noCacheTask(
    steps(
      runTask("gen:schema"),
      "pnpm --dir npm/cli exec json2ts -i schemas/vize.config.schema.json -o src/types/generated.ts",
      nodeTool("tasks/postprocess-types"),
      nodeTool("tasks/generate-vue-config-validation"),
      "pnpm --dir npm/cli exec oxfmt --write schemas/vize.config.schema.json src/types/generated.ts ../../crates/vize_carton/src/config/loader/vite-runtime.mjs",
    ),
  ),
  gen: noCacheTask(runTask("gen:types")),
  cli: noCacheTask(nodeTool("tasks/install-cli")),
  "cli:help": noCacheTask("vize --help"),
  "cli:example": noCacheTask("vize './**/*.vue' -o . -v"),
  "cli:example-json": noCacheTask("vize './**/*.vue' -o . -f json -v"),
  "cli:example-ssr": noCacheTask("vize './**/*.vue' -o . -f json --ssr -v"),
  "cli:example-stats": noCacheTask("vize './**/*.vue' -f stats -v"),
});
