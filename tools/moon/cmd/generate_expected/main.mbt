
async fn main {
  let exit_code = @process.run(
    "node",
    ["tests/tooling/support/generate-expected.ts"] + @tool.cli_args(),
  )
  @sys.exit(exit_code)
}
