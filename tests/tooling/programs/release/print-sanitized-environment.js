const names = ["GH_TOKEN", "GITHUB_TOKEN", "NODE_AUTH_TOKEN", "NPM_TOKEN", "CORSA_PATH"];
process.stdout.write(JSON.stringify(names.map((name) => process.env[name])));
