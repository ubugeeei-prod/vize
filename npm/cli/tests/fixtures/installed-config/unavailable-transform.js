import { appendFileSync } from "node:fs";
appendFileSync("binding-loads.txt", "oxc-transform\n");
throw new Error("oxc-transform: deliberately unavailable binding");
