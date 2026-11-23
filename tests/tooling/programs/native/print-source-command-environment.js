process.stdout.write(
  JSON.stringify([
    process.cwd(),
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH ?? null,
    process.env.VIZE_PREFER_WORKSPACE_BINDING,
    process.env.NODE_OPTIONS ?? null,
  ]),
);
process.stderr.write("whole stderr\n");
