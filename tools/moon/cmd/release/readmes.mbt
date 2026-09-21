///|
/// A published benchmark is evidence about the measured binary, not the
/// next release. Keep marked publication sections byte-for-byte unchanged.
fn replace_readme_version_references(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let lines : Array[String] = []
  let mut benchmark = false
  for view in content.split("\n") {
    let line = view.to_owned()
    if line.contains("<!-- benchmark:readme:start -->") {
      benchmark = true
    }
    lines.push(
      if benchmark {
        line
      } else {
        line.replace_all(old=current_version, new=new_version)
      },
    )
    if line.contains("<!-- benchmark:readme:end -->") {
      benchmark = false
    }
  }
  lines.join("\n")
}

///|
async fn update_release_readmes(
  current_version : String,
  new_version : String,
  files_to_stage : Array[String],
) -> Bool {
  println("Updating READMEs...")
  let root_readme = "README.md"
  if @tool.path_exists(root_readme) && !is_git_ignored(root_readme) {
    let content = match @tool.read_file_to_string(root_readme) {
      Some(content) => content
      None => {
        @stdio.stderr.write("Failed to read \{root_readme}\n")
        return false
      }
    }
    if !write_string_to_file(
        root_readme,
        replace_readme_version_references(content, current_version, new_version),
      ) {
      @stdio.stderr.write("Failed to update \{root_readme}\n")
      return false
    }
    files_to_stage.push(root_readme)
  }
  let readme_package_dirs : Array[String] = []
  for entry_name in @tool.read_dir("npm") {
    let package_dir = @tool.join_path("npm", entry_name)
    if !@tool.is_dir(package_dir) {
      continue
    }
    readme_package_dirs.push(package_dir)
  }
  for package_dir in nested_npm_package_dirs() {
    readme_package_dirs.push(package_dir)
  }
  for package_dir in readme_package_dirs {
    let readme_path = @tool.join_path(package_dir, "README.md")
    if !@tool.path_exists(readme_path) {
      continue
    }
    if is_git_ignored(readme_path) {
      continue
    }
    let content = match @tool.read_file_to_string(readme_path) {
      Some(content) => content
      None => {
        @stdio.stderr.write("Failed to read \{readme_path}\n")
        return false
      }
    }
    if !write_string_to_file(
        readme_path,
        replace_readme_version_references(content, current_version, new_version),
      ) {
      @stdio.stderr.write("Failed to update \{readme_path}\n")
      return false
    }
    files_to_stage.push(readme_path)
  }

  true
}
