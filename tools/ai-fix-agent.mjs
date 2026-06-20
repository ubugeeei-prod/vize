#!/usr/bin/env node

import { main } from "./ai-fix-agent/cli.mjs";

main(process.argv.slice(2)).catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
