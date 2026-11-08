import { runEntrypoint } from "../../../support/command-runtime.ts";
import { main } from "../../../support/compat/github/app-e2e-plan.ts";

runEntrypoint(() => {
  main(process.argv.slice(2));
  return 0;
});
