.verification_results as $report
| ($report.summary | .status == "completed" and .executed == 2 and .successful == 2)
  and ([$report.results[].harness_id | split("::") | last] | sort == [
    "original_source_block_preserves_hole_positions_and_identity",
    "original_source_span_preserves_the_selected_bytes"
  ])
  and all($report.results[];
    .status == "Success" and
    ([.checks[] | select(.category == "cover")] |
      length == 3 and all(.[]; .status == "Satisfied")))
