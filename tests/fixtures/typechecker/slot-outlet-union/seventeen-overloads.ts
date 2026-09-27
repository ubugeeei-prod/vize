// Prepend the generated __VizeSlotPayload* and __VizeIsAny helpers.
// Both assignments should be accepted. The current sixteen-signature helper
// incorrectly drops `first`, producing TS2322 on the first assignment.
type SeventeenOutlets = {
  (props: { kind: "first" }): any;
  (props: { kind: "outlet1" }): any;
  (props: { kind: "outlet2" }): any;
  (props: { kind: "outlet3" }): any;
  (props: { kind: "outlet4" }): any;
  (props: { kind: "outlet5" }): any;
  (props: { kind: "outlet6" }): any;
  (props: { kind: "outlet7" }): any;
  (props: { kind: "outlet8" }): any;
  (props: { kind: "outlet9" }): any;
  (props: { kind: "outlet10" }): any;
  (props: { kind: "outlet11" }): any;
  (props: { kind: "outlet12" }): any;
  (props: { kind: "outlet13" }): any;
  (props: { kind: "outlet14" }): any;
  (props: { kind: "outlet15" }): any;
  (props: { kind: "last" }): any;
};
type Payload = __VizeSlotPayloadUnify<__VizeSlotPayloadOf<SeventeenOutlets>>;
const first: Payload = { kind: "first" };
const last: Payload = { kind: "last" };
