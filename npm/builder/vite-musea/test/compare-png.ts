import { compareImages } from "../src/vrt/runner-comparison.ts";

const [baseline, current, diff] = process.argv.slice(2);
try {
  await compareImages(baseline, current, diff, {});
  process.stdout.write(JSON.stringify({ rejected: false }));
} catch (error) {
  if (!(error instanceof Error)) throw error;
  process.stdout.write(
    JSON.stringify({
      rejected: true,
      code: "code" in error ? error.code : null,
      message: error.message,
    }),
  );
}
