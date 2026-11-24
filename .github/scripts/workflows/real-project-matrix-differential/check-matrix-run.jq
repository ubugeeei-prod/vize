.path == ".github/workflows/real-project-matrix.yml"
and .head_branch == "main"
and .status == "completed"
and (.display_title | contains("scoped-diagnostic") | not)
