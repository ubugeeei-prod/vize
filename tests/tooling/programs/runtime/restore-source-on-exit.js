import { installSourceRestores } from "../../../app/dev/source-restore.ts";

const [entries, mode] = process.argv.slice(2);
installSourceRestores(JSON.parse(entries));
process.stdout.write("ready\n");
if (mode === "exit") setTimeout(() => process.exit(0), 50);
else setInterval(() => {}, 1000);
