import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  archive,
  action,
  original,
  primary,
  alternative,
  upstream,
  digest,
  fixture,
  primaryArguments,
  assertBeforeExtraction,
} from "./fixtures/setup-wild-recipe.mjs";

void test("Wild adapter reverse expansion retains the whole pinned original action", () => {
  assert.equal(
    digest(original),
    "80d1b927f19f7df0e00d99b44361104c13f2221d630bbd080f7dc6eb066fe61a",
  );
  const before = parse(original),
    after = parse(action),
    delegate = after.runs.steps.shift();
  assert.equal(delegate.uses, upstream);
  assert.deepEqual(delegate.with, {
    "wild-version": "${{ inputs.wild-version }}",
    "download-retries": "${{ inputs.download-retries }}",
  });
  const step = after.runs.steps[0];
  step.if = before.runs.steps[0].if;
  step.run = step.run.replaceAll(
    "/0.9.0/wild-linker-0.9.0-",
    "/${{ inputs.wild-version }}/wild-linker-${{ inputs.wild-version }}-",
  );
  const sourceLine = ' . "$GITHUB_ACTION_PATH/download-linux-x64.sh" "${archive}" "${url}" "5"';
  const originalDownload = before.runs.steps[0].run.match(
    /curl --fail-with-body[\s\S]*?"\$\{url\}"/,
  )[0];
  assert.equal(step.run.split(sourceLine.trim()).length, 2);
  step.run = step.run.replace(sourceLine.trim(), originalDownload);
  assert.deepEqual(after, before);
});
void test("Wild delegation is complementary for every unsupported host or input", () => {
  const steps = parse(action).runs.steps;
  assert.equal(
    steps[0].if,
    "${{ runner.os != 'Linux' || runner.arch != 'X64' || inputs.wild-version != '0.9.0' || inputs.download-retries != '5' }}",
  );
  assert.equal(
    steps[1].if,
    "${{ runner.os == 'Linux' && runner.arch == 'X64' && inputs.wild-version == '0.9.0' && inputs.download-retries == '5' }}",
  );
  function applies(expression, values) {
    return expression
      .slice(4, -3)
      .split(" || ")
      .some((group) =>
        group.split(" && ").every((part) => {
          const match =
            /^(runner\.(?:os|arch)|inputs\.(?:wild-version|download-retries)) (==|!=) '([^']+)'$/.exec(
              part,
            );
          assert.ok(match);
          const equal = values[match[1]].toLowerCase() === match[3].toLowerCase();
          return match[2] === "==" ? equal : !equal;
        }),
      );
  }
  for (const system of ["Linux", "Windows", "macOS"])
    for (const arch of ["X64", "ARM64", "other"])
      for (const version of ["0.9.0", "0.9.1"])
        for (const retries of ["5", "0", "6"]) {
          const owned =
            system === "Linux" && arch === "X64" && version === "0.9.0" && retries === "5";
          const values = {
            "runner.os": system,
            "runner.arch": arch,
            "inputs.wild-version": version,
            "inputs.download-retries": retries,
          };
          assert.equal(applies(steps[1].if, values), owned);
          assert.equal(applies(steps[0].if, values), !owned);
        }
});
void test("Only Playground and canonical SSR-Pug use the new adapter", () => {
  const pr = parse(fs.readFileSync(".github/workflows/pr-source-checks.yml", "utf8"));
  const local = "./.github/actions/setup-wild-http-recovery";
  assert.equal(pr.jobs["pr-playground-test"].steps.filter((s) => s.uses === local).length, 1);
  const uses = (jobs, action) =>
    Object.entries(jobs).flatMap(([name, job]) =>
      (job.steps ?? []).filter((s) => s.uses === action).map(() => name),
    );
  // Every other Wild user keeps the pinned upstream action, counted per file.
  assert.deepEqual(uses(pr.jobs, local), ["pr-playground-test"]);
  assert.deepEqual(uses(pr.jobs, upstream), ["pr-native-runtime"]);
  const js = parse(fs.readFileSync(".github/workflows/pr-js-checks.yml", "utf8"));
  assert.deepEqual(uses(js.jobs, local), []);
  assert.deepEqual(uses(js.jobs, upstream), Object.keys(js.jobs));
  const canonical = parse(
    fs.readFileSync(".github/workflows/davinci-canonical-corpus.yml", "utf8"),
  );
  const steps = canonical.jobs["canonical-observers"].steps;
  assert.deepEqual(
    steps.find((s) => s.uses === local),
    {
      name: "Setup Wild linker recovery (SSR-Pug)",
      if: "${{ matrix.observer == 'ssr-pug' }}",
      uses: local,
      with: { "wild-version": "0.9.0" },
    },
  );
  assert.equal(steps.find((s) => s.uses === upstream).if, "${{ matrix.observer != 'ssr-pug' }}");
});
void test("Expanded primary success equals original stdout/stderr/files/config and removes archive", () =>
  fixture(({ run }) => {
    for (const config of [
      undefined,
      "# user config\n[target.x86_64-unknown-linux-gnu]\nlinker = 'user'\n",
      "[target.x86_64-unknown-linux-gnu]\nlinker='gnu'\n[target.x86_64-unknown-linux-musl]\nlinker='musl'\n",
    ]) {
      const before = run({ original: true, config }),
        after = run({ config });
      assert.equal(before.status, 0);
      assert.equal(after.status, 0);
      assert.deepEqual({ ...after, calls: [] }, { ...before, calls: [] });
      assert.deepEqual(before.calls[0], primaryArguments(before.calls[0][7], false));
      assert.deepEqual(after.calls[0], primaryArguments(after.calls[0][7]));
      assert.equal(
        after.files.some((r) => r.path === "temporary/wild.tar.gz"),
        false,
      );
    }
  }));
for (const http of ["500", "502", "503", "504"])
  void test(`Expanded HTTP${http} recovery extracts identical files and config`, () =>
    fixture(({ run }) => {
      assertBeforeExtraction(
        run({ original: true, env: { PRIMARY_EXIT: "22", HTTP: http, PRIMARY_BODY: "wrong" } }),
        22,
        1,
      );
      const before = run({ original: true }),
        after = run({ env: { PRIMARY_EXIT: "22", HTTP: http, PRIMARY_BODY: "wrong" } });
      assert.equal(after.status, 0);
      assert.deepEqual(after.files, before.files);
      assert.deepEqual(after.tarCalls, before.tarCalls);
      assert.equal(after.stdout, before.stdout);
      assert.equal(after.stderr, "primary fixture stderr\napi fixture stderr\n");
      assert.deepEqual(after.calls[0], primaryArguments(after.calls[0][7]));
      assert.deepEqual(after.calls[1], [
        "--fail-with-body",
        "--show-error",
        "--location",
        "--header",
        "Accept: application/octet-stream",
        "--header",
        "X-GitHub-Api-Version: 2022-11-28",
        "--output",
        after.calls[0][7],
        alternative,
      ]);
    }));
void test("Existing install directory retains original mkdir refusal before any download", () =>
  fixture(({ run }) => {
    const before = run({ original: true, existing: true }),
      after = run({ existing: true });
    assert.equal(after.status, 1);
    assert.deepEqual(after, before);
    assert.deepEqual(after.calls, []);
    assert.deepEqual(after.tarCalls, []);
  }));
void test("Primary authorization, rate, TLS and malformed status failures never use API", () =>
  fixture(({ run }) => {
    for (const http of [
      "401",
      "403",
      "404",
      "429",
      "503\n",
      "0503",
      "5O3",
      "503suffix",
      "503|",
      "",
    ]) {
      assertBeforeExtraction(run({ env: { PRIMARY_EXIT: "22", HTTP: http } }), 22, 1);
    }
    for (const status of ["6", "7", "28", "35"])
      assertBeforeExtraction(
        run({ env: { PRIMARY_EXIT: status, HTTP: "504" } }),
        Number(status),
        1,
      );
    assertBeforeExtraction(run({ env: { PRIMARY_EXIT: "0", HTTP: "503" } }), 1, 1);
  }));
void test("Failed API status and wrong complete or truncated body refuse before original tar", () =>
  fixture(({ run, root }) => {
    for (const status of ["22", "28"])
      assertBeforeExtraction(
        run({ env: { PRIMARY_EXIT: "22", HTTP: "504", API_EXIT: status } }),
        Number(status),
        2,
      );
    assertBeforeExtraction(
      run({ env: { PRIMARY_EXIT: "22", HTTP: "504", API_BODY: "wrong" } }),
      1,
      2,
    );
    const payload = path.join(root, "payload");
    fs.writeFileSync(payload, fs.readFileSync(payload).subarray(0, 25));
    assertBeforeExtraction(run({ env: { PRIMARY_EXIT: "22", HTTP: "504" } }), 1, 2);
  }));
void test("Changed member bytes or mode and primary wrong body fail whole archive SHA", () =>
  fixture(({ run, root }) => {
    assertBeforeExtraction(run({ env: { PRIMARY_BODY: "wrong" } }), 1, 1);
    for (const bytes of [archive("different inert member\n"), archive(undefined, "0000755\0")]) {
      fs.writeFileSync(path.join(root, "payload"), bytes);
      assertBeforeExtraction(run(), 1, 1);
    }
  }));
void test("Exact context refusal and API nonzero status do not depend on inherited errexit", () =>
  fixture(({ root, local, bytes }) => {
    const url = primary,
      env = {
        ...process.env,
        PATH: `${path.join(root, "bin")}${path.delimiter}${process.env.PATH}`,
        RUNNER_OS: "Linux",
        RUNNER_ARCH: "X64",
        CALLS: path.join(root, "calls"),
        PAYLOAD: path.join(root, "payload"),
        PRIMARY_BODY: "valid",
        API_BODY: "valid",
        PRIMARY_EXIT: "22",
        HTTP: "504",
        API_EXIT: "28",
      };
    for (const [change, requested, retry] of [
      [{}, url, "5"],
      [{ RUNNER_OS: "Windows" }, url, "5"],
      [{ RUNNER_ARCH: "ARM64" }, url, "5"],
      [{}, `${url}?other`, "5"],
      [{}, url, "6"],
    ]) {
      fs.writeFileSync(env.CALLS, "");
      fs.writeFileSync(env.PAYLOAD, bytes);
      const p = spawnSync(
        "bash",
        [
          "-c",
          'set +e; . "$1" "$2" "$3" "$4"; printf AFTER_HELPER',
          "law",
          path.join(local, "download-linux-x64.sh"),
          path.join(root, "archive"),
          requested,
          retry,
        ],
        { env: { ...env, ...change }, encoding: "utf8" },
      );
      assert.equal(
        p.status,
        Object.keys(change).length === 0 && requested === url && retry === "5" ? 28 : 1,
      );
      assert.equal(p.stdout.includes("AFTER_HELPER"), false);
    }
  }));
