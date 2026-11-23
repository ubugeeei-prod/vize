import { pathToFileURL } from "node:url";
const { default: get } = await import(pathToFileURL(process.argv[2]).href);
process.stdout.write(get());
