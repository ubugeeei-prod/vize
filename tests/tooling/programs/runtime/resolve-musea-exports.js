const [packageUrl] = process.argv.slice(2);
const paths = ["@vizejs/musea-nuxt", "@vizejs/musea-nuxt/auto-imports"];
console.log(JSON.stringify(paths.map((name) => import.meta.resolve(name, packageUrl))));
