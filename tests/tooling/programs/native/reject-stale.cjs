function causeMessages(error) {
  const messages = [];
  let current = error;
  while (current) {
    messages.push(current.message);
    current = current.cause;
  }
  return messages.join("\n");
}

try {
  require("@vizejs/native");
  throw new Error("stale binding unexpectedly loaded");
} catch (error) {
  const messages = causeMessages(error);
  if (
    !messages.includes("Native binding package version mismatch") ||
    !messages.includes("expected " + process.argv[2] + " but got " + process.argv[3])
  ) {
    console.error(messages);
    process.exit(1);
  }
  console.log("package root rejected stale binding");
}
