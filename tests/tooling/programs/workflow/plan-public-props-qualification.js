import { planToolingTests } from "../../../../tools/support/compat/github/plan-tooling-tests.mjs";

const requests = JSON.parse(process.argv[2]);
console.log(
  JSON.stringify(
    requests.map(
      ({ input, tier }) => planToolingTests([input], { tier, cwd: process.cwd() }).tests,
    ),
  ),
);
