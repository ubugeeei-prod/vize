select(
  .reason == "compiler-artifact" and .target.name == "vize"
  and .profile.test == true and .executable != null
)
| .executable
