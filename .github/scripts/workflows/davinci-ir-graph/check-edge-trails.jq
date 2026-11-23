.commands
| keys == ["DistinctSelfLoops", "ExactTrails", "ParallelReturn", "SingleEdgeReturnsOnlyOnReuse"]
  and (.ExactTrails | .type == "check" and (.solution | length) == 0)
  and ([.DistinctSelfLoops, .ParallelReturn, .SingleEdgeReturnsOnlyOnReuse]
       | all(.type == "run" and (.solution | length) == 1))
