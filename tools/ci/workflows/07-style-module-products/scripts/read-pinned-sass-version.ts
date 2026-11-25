import { readFileSync } from "node:fs";

const manifest = JSON.parse(readFileSync("npm/builder/parcel/package.json", "utf8")) as {
  devDependencies: { sass: string };
};
console.log(manifest.devDependencies.sass);
