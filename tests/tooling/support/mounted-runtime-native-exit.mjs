import { writeSync } from "node:fs";
import { leaveMountedRuntimeProcess } from "./davinci-mounted-trace.mjs";

// Importing the mounted trace loads vite-plus's native binding. The parent
// fails unless this process exits 0 with the passing payload.
leaveMountedRuntimeProcess();
writeSync(1, '{"passed":1}');
