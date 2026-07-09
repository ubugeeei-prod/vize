
fn npm_tag_for_version(version : String) -> String {
  if version.contains("-alpha") {
    "alpha"
  } else if version.contains("-beta") {
    "beta"
  } else if version.contains("-rc") {
    "rc"
  } else {
    "latest"
  }
}

fn main {
  let args = @tool.cli_args()
  if args.is_empty() {
    println("Usage: moon run --target native tools/moon/cmd/npm_tag -- <version>")
    @sys.exit(1)
  }
  println(npm_tag_for_version(args[0]))
}
