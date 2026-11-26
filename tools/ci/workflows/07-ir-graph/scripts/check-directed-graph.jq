.commands
| keys == ["DisconnectedCycle", "Exact", "MissedSuccessor", "ParallelCycle"]
  and (.Exact | .type == "check" and (.solution | length) == 0)
  and ([.DisconnectedCycle, .MissedSuccessor, .ParallelCycle]
       | all(.type == "run" and (.solution | length) == 1))
