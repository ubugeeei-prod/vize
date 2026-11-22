--------------------------- MODULE Publication ---------------------------
EXTENDS Naturals
CONSTANT Guarded
VARIABLES rev, document, body, offset, ticket, published, memo
vars == <<rev, document, body, offset, ticket, published, memo>>
Bodies == {"one", "two", "broken"}
No == 0
NoTicket == [r |-> 3, d |-> 0, b |-> "broken", o |-> 0]
NoPublication == [r |-> 3, d |-> 0, value |-> No, pos |-> 0]
Answer(b) == CASE b = "one" -> 1 [] b = "two" -> 2 [] OTHER -> No
Current == [r |-> rev, d |-> document, b |-> body, o |-> offset]
Init == /\ rev = 0 /\ document = 0 /\ body = "one" /\ offset = 0
        /\ ticket = NoTicket /\ published = NoPublication /\ memo = [b \in Bodies |-> No]
Begin == /\ ticket' = Current /\ UNCHANGED <<rev, document, body, offset, published, memo>>
Compute == /\ ticket # NoTicket
           /\ memo' = [memo EXCEPT ![ticket.b] = Answer(ticket.b)]
           /\ UNCHANGED <<rev, document, body, offset, ticket, published>>
Edit == /\ rev < 2 /\ rev' = rev + 1
        /\ document' \in 0..1 /\ body' \in Bodies /\ offset' \in 0..1
        /\ published' = NoPublication /\ UNCHANGED <<ticket, memo>>
Publish == /\ ticket # NoTicket /\ memo[ticket.b] # No
           /\ (~Guarded \/ ticket = Current)
           /\ published' = [r |-> ticket.r, d |-> ticket.d, value |-> memo[ticket.b], pos |-> ticket.o + 1]
           /\ UNCHANGED <<rev, document, body, offset, ticket, memo>>
Next == Begin \/ Compute \/ Edit \/ Publish
Spec == Init /\ [][Next]_vars
CurrentAnswer == published = NoPublication \/
                 /\ published.r = rev /\ published.d = document
                 /\ published.value = Answer(body) /\ published.pos = offset + 1
MemoSound == \A b \in Bodies: memo[b] = No \/ memo[b] = Answer(b)
=============================================================================
