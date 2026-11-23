process.stdout.write(
  JSON.stringify([
    process.env.CORSA_PATH,
    process.env.VIZE_TEST_REQUIRE_TSGO,
    process.env.VIZE_TEST_DISABLE_TSGO,
  ]),
);
