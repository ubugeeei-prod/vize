/// Recognizes npm's staged-version conflict through package-manager wrapping.
/// Normalize only the diagnostic view; preserve the original process streams.
fn is_staged_publish_error(detail : String) -> Bool {
  let wrapped = detail
    .replace_all(old="\r", new=" ")
    .replace_all(old="\n", new=" ")
    .replace_all(old="\t", new=" ")
    .replace_all(old="│", new=" ")
  let words : Array[String] = []
  for word in wrapped.split(" ") {
    if word != "" {
      words.push(word.to_owned())
    }
  }
  words.join(" ").contains("Cannot publish over previously staged version")
}
