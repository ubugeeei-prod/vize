module.exports = () => {
  if (process.argv.includes("--sdk")) process.exit(1);
};
