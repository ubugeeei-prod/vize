// CI explicitly prepares the pinned isolation runtime; failures stop the gate.
import { execFileSync } from "node:child_process";
import { SANDBOX_IMAGE } from "../../../../npm/plugin-sdk/sandbox.js";

execFileSync("docker", ["pull", SANDBOX_IMAGE], { stdio: "inherit", timeout: 180000 });
