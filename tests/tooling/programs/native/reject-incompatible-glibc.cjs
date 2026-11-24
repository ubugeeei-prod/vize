try {
  require("@vizejs/native");
  throw new Error("broken binding unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("GLIBC_2.39")) {
    console.error(error.message);
    process.exit(1);
  }
  if (error.message.includes("optional dependencies")) {
    console.error(error.message);
    process.exit(1);
  }
  // A glibc mismatch reports what the binding asked for and the supported floor.
  for (const hint of [
    "the binding asks for GLIBC_2.39.",
    "built for glibc 2.17 or newer",
    "musl build",
  ]) {
    if (!error.message.includes(hint)) {
      console.error(error.message);
      process.exit(1);
    }
  }
  if (error.cause?.code !== "ERR_DLOPEN_FAILED") {
    console.error(error.cause);
    process.exit(1);
  }
  console.log("package root surfaced native loader failure");
}
