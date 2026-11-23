.commands
| keys == ["LaterWrite", "NonemptyTrace", "RepeatedName", "SameTrace"]
  and (.SameTrace | .type == "check" and (.solution | length) == 0)
  and ([.LaterWrite, .NonemptyTrace, .RepeatedName]
       | all(.type == "run" and (.solution | length) == 1))
