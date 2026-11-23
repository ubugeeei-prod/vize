import { pathToFileURL } from "node:url";

const { TODO_LIST } = await import(pathToFileURL(process.argv[2]).href);
process.stdout.write(TODO_LIST);
