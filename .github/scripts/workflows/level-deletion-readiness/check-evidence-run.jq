.head_sha == $sha
and .conclusion == "success"
and .path == ".github/workflows/real-project-matrix.yml"
and (.event == "schedule" or .event == "workflow_dispatch")
