const [name, consumer] = process.argv.slice(2);
process.stdout.write(require.resolve(name, { paths: [consumer] }));
