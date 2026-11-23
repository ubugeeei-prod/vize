try {
  require("@vizejs/native");
  throw new Error("missing binding unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("optional dependencies")) {
    console.error(error.message);
    process.exit(1);
  }
  // Name the absent platform package instead of only blaming npm.
  const absent = error.message.split("\n")[1] ?? "";
  if (!absent.includes(process.argv[2]) || !absent.includes(" is not installed (")) {
    console.error(error.message);
    process.exit(1);
  }
  console.log("package root reported missing optional dependency");
}
