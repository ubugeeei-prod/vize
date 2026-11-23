try {
  require("@vizejs/native");
  throw new Error("broken native package unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("missing-internal-native-helper")) process.exit(1);
  if (error.message.includes("optional dependencies")) process.exit(1);
  console.log("package root surfaced missing internal dependency");
}
