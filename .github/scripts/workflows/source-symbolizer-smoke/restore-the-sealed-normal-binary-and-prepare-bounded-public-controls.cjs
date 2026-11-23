process.stdout.write(
  JSON.stringify({
    private: true,
    dependencies: {
      vue: "3.6.0-rc.13",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
    },
  }),
);
