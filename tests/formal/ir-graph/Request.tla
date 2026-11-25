------------------------------ MODULE Request ------------------------------
EXTENDS Naturals
CONSTANT Policy
VARIABLES lastRevision, document, captured, phase, cancelled, template, style, delivery
vars == <<lastRevision, document, captured, phase, cancelled, template, style, delivery>>

Markup == {"complete", "broken"}
Styles == {"inline", "external"}
Doc(r, v, m, s) == [revision |-> r, version |-> v, markup |-> m, css |-> s]
Closed == Doc(0, 0, "broken", "external")
Answers(d) == [template |-> IF d.markup = "broken" THEN "syntax error" ELSE "syntax",
               style |-> IF d.css = "inline" THEN "compiled" ELSE "external"]
Current == document.revision > 0 /\ document = captured

Init == /\ lastRevision = 1 /\ document = Doc(1, 1, "complete", "inline")
        /\ captured = Closed /\ phase = "idle" /\ cancelled = FALSE
        /\ template = "pending" /\ style = "pending"
        /\ delivery = [document |-> Closed, cancelled |-> FALSE]

\* One URI. Reopening can reuse both its client version and its source text.
Change(opening) ==
    /\ lastRevision < 3 /\ (document.revision = 0) = opening
    /\ lastRevision' = lastRevision + 1
    /\ \E m \in Markup, s \in Styles:
         document' = Doc(lastRevision', IF opening THEN 1 ELSE document.version + 1, m, s)
    /\ UNCHANGED <<captured, phase, cancelled, template, style, delivery>>
Close == /\ document.revision > 0 /\ document' = Closed
         /\ UNCHANGED <<lastRevision, captured, phase, cancelled, template, style, delivery>>
Begin == /\ phase = "idle" /\ document.revision > 0
         /\ captured' = document /\ phase' = "captured"
         /\ UNCHANGED <<lastRevision, document, cancelled, template, style, delivery>>
Cancel == /\ phase \in {"captured", "running", "template read", "ready"} /\ ~cancelled
          /\ cancelled' = TRUE
          /\ UNCHANGED <<lastRevision, document, captured, phase, template, style, delivery>>

Start == /\ phase = "captured" /\ Current /\ ~cancelled /\ phase' = "running"
         /\ UNCHANGED <<lastRevision, document, captured, cancelled, template, style, delivery>>
ReadTemplate == /\ phase = "running" /\ template' = Answers(captured).template
                /\ phase' = "template read"
                /\ UNCHANGED <<lastRevision, document, captured, cancelled, style, delivery>>
Finish == /\ phase = "template read" /\ Current /\ ~cancelled
          /\ style' = Answers(captured).style /\ phase' = "ready"
          /\ UNCHANGED <<lastRevision, document, captured, cancelled, template, delivery>>

\* Record freshness at the host's guarded publication. Later edits leave that
\* record intact. Network delivery is outside the read guard.
Publish == /\ phase = "ready" /\ document.revision > 0
           /\ document.version = captured.version
           /\ (Policy = "client version only" \/ document.revision = captured.revision)
           /\ (Policy = "ignore late cancellation" \/ ~cancelled)
           /\ delivery' = [document |-> document, cancelled |-> cancelled]
           /\ phase' = "published"
           /\ UNCHANGED <<lastRevision, document, captured, cancelled, template, style>>

Next == Change(FALSE) \/ Close \/ Change(TRUE) \/ Begin \/ Cancel \/ Start \/ ReadTemplate
        \/ Finish \/ Publish
Spec == Init /\ [][Next]_vars
CurrentRevision == phase = "published" => delivery.document = captured
LiveRequest == phase = "published" => ~delivery.cancelled
OriginalOutputs == phase = "published" =>
                   [template |-> template, style |-> style] = Answers(delivery.document)
NoPartialOutput == ~(phase = "published" /\ template = "syntax error" /\ style = "compiled")
=============================================================================
