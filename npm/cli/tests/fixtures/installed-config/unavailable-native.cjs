const { appendFileSync } = require("node:fs");
appendFileSync("binding-loads.txt", "@vizejs/native\n");
throw new Error("@vizejs/native: deliberately unavailable binding");
