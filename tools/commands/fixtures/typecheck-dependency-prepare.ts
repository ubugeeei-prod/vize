import { runEntrypoint } from "../../support/command-runtime.ts";
import { runTypecheckDependencyPrepare } from "../../support/compat/fixtures/typecheck-dependency-prepare.mjs";

runEntrypoint(async () => {
  await runTypecheckDependencyPrepare();
});
