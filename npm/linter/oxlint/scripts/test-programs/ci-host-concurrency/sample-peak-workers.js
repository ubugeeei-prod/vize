import fs from "node:fs";
const [directory, id, failure] = process.argv.slice(2);
fs.writeFileSync(directory + "/start.json", JSON.stringify({ time: Date.now(), pid: process.pid }));
console.log("started " + id);
console.error("evidence " + id);
await new Promise((resolve) => setTimeout(resolve, 200));
fs.writeFileSync(directory + "/end.json", JSON.stringify({ time: Date.now() }));
process.exitCode = Number(failure);
