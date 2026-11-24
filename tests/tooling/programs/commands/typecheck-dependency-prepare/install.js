fs.mkdirSync("node_modules", { recursive: true });
fs.writeFileSync("node_modules/installed", "yes");
process.stdout.write("installed");
