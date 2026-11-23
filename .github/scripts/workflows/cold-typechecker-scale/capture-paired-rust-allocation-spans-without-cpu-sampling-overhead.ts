import fs from "node:fs";
for (const source of ["base", "head"]) {
  const x = JSON.parse(
    fs.readFileSync(
      process.env.RUNNER_TEMP + "/cold-scale/profile-" + source + "/profile.json",
      "utf8",
    ),
  );
  if (!x.allocation) throw Error("allocation instrumentation missing");
}
