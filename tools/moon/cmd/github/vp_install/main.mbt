
async fn main {
  let exit_code = @process.run(
    "vp",
    ["install", "--frozen-lockfile", "--prefer-offline"] + @tool.cli_args(),
  )
  @sys.exit(exit_code)
}
