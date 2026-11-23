function hasCauseCode(error, code) {
  let current = error;
  while (current) {
    if (current.code === code) return true;
    current = current.cause;
  }
  return false;
}

try {
  require("@vizejs/native");
  throw new Error("broken native package unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("Failed to load native binding:")) process.exit(1);
  if (!error.message.includes("GLIBC_2.39")) process.exit(1);
  if (error.message.includes("optional dependencies")) process.exit(1);
  if (!hasCauseCode(error, "ERR_DLOPEN_FAILED")) process.exit(1);
  console.log("package root surfaced dlopen failure");
}
