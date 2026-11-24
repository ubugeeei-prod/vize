-------------------------- MODULE TemplateRevision --------------------------
EXTENDS Naturals, Sequences

CONSTANT Fault
Bodies == {"first", "changed", "damaged"}
Sources == Bodies \cup {"missing", "ambiguous"}
Bases == {10, 19, 32}

\* Each retained row has an ordered identity and a block-relative byte range.
Rows(body) ==
  CASE body = "first" -> <<<<"if", 1, 7>>, <<"logical", 3, 5>>>>
    [] body = "changed" -> <<<<"for", 2, 4>>>>
    [] body = "damaged" -> <<<<"diagnostic", 3, 3>>>>
    [] OTHER -> <<>>
At(rows, start) == [i \in 1..Len(rows) |->
    <<rows[i][1], start + rows[i][2], start + rows[i][3]>>]
Direct(source, start) ==
  IF source \in Bodies THEN <<"present", At(Rows(source), start)>>
  ELSE <<source, <<>>>>

VARIABLES source, base, key, cachedBase, visible, answer
vars == <<source, base, key, cachedBase, visible, answer>>
Empty == <<"missing", <<>>>>
Init == /\ source = "missing" /\ base = 10 /\ key = "none"
        /\ cachedBase = 10 /\ visible = FALSE /\ answer = Empty

Edit == /\ source' \in Sources /\ base' \in Bases
        /\ visible' = FALSE /\ UNCHANGED <<key, cachedBase, answer>>
Close == /\ source' = "missing" /\ base' = 10 /\ key' = "none"
         /\ cachedBase' = 10 /\ visible' = FALSE /\ answer' = Empty
Read ==
  LET selected == IF Fault = "admitAmbiguous" /\ source = "ambiguous" THEN "first" ELSE source
      reuse == key \in Bodies /\ (key = selected \/ Fault = "ignoreContent")
      body == IF reuse THEN key ELSE selected
      origin == IF reuse /\ Fault = "reuseBase" THEN cachedBase ELSE base
  IN /\ visible' = TRUE
     /\ IF selected \in Bodies THEN
           /\ key' = body /\ cachedBase' = origin
           /\ answer' = <<"present", At(Rows(body), origin)>>
        ELSE /\ key' = "none" /\ cachedBase' = base /\ answer' = Direct(selected, base)
     /\ UNCHANGED <<source, base>>

Next == Edit \/ Read \/ Close
Spec == Init /\ [][Next]_vars
ExactReport == ~visible \/ answer = Direct(source, base)
=============================================================================
