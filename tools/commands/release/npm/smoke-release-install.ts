import { runEntrypoint } from "../../../support/command-runtime.ts";
import { smokeReleaseInstall } from "../../../support/release/smoke_release_install/main.ts";
runEntrypoint(() => smokeReleaseInstall(process.argv.slice(2)));
