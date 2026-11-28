import Foundation
import XCTest

@testable import NativePacket

final class NativeBridgePacketTests: XCTestCase {
  private func counter(sequence: Int = 1) -> [String: Any] {
    packet(
      [
        ["id": 1, "kind": "view", "children": [2, 3], "gap": 8, "padding": 12],
        ["id": 2, "kind": "text", "children": [], "text": "Count: 0"],
        ["id": 3, "kind": "button", "children": [], "label": "Increment"],
      ], sequence: sequence)
  }

  private func packet(_ nodes: [[String: Any]], sequence: Int = 1, root: Int = 1) -> [String: Any] {
    [
      "format": "vize.native-view-tree", "formatVersion": 1, "sequence": sequence, "root": root,
      "nodes": nodes,
    ]
  }

  private func reject(
    _ value: Any, code: String, after: Int = 0, file: StaticString = #filePath, line: UInt = #line
  ) {
    do {
      _ = try NativeBridgeParser.parse(value, after: after)
      XCTFail("expected refusal \(code)", file: file, line: line)
    } catch let failure as NativeBridgeFailure {
      XCTAssertEqual(failure.code, code, file: file, line: line)
    } catch {
      XCTFail("expected a stable bridge refusal", file: file, line: line)
    }
  }

  func testAcceptsTheCounterWithoutMutatingItsPacket() throws {
    let input = counter()
    let before = try JSONSerialization.data(withJSONObject: input, options: [.sortedKeys])
    guard case .render(let packet) = try NativeBridgeParser.parse(input) else {
      return XCTFail("expected a view tree")
    }
    XCTAssertEqual(packet.sequence, 1)
    XCTAssertEqual(packet.root, 1)
    XCTAssertEqual(packet.nodes.count, 3)
    XCTAssertEqual(packet.nodes[1]?.gap, 8)
    XCTAssertEqual(packet.nodes[1]?.padding, 12)
    XCTAssertEqual(packet.nodes[3]?.label, "Increment")
    XCTAssertEqual(
      try JSONSerialization.data(withJSONObject: input, options: [.sortedKeys]), before)
  }

  func testAcceptsOnlyAdvancingSnapshots() throws {
    reject(counter(), code: "stale-sequence", after: 1)
    reject(counter(), code: "stale-sequence", after: 2)
    guard case .render(let packet) = try NativeBridgeParser.parse(counter(sequence: 2), after: 1)
    else {
      return XCTFail("expected the next snapshot")
    }
    XCTAssertEqual(packet.sequence, 2)
  }

  func testRetainsCapabilityDiagnosticsWithoutInstallingAViewTree() throws {
    let value: [String: Any] = [
      "format": "vize.native-diagnostic", "formatVersion": 1,
      "code": "missing-capability", "message": "native.images",
    ]
    guard case .diagnostic(let diagnostic) = try NativeBridgeParser.parse(value) else {
      return XCTFail("expected a diagnostic")
    }
    XCTAssertEqual(diagnostic.code, "missing-capability")
    XCTAssertEqual(diagnostic.message, "native.images")
  }

  func testRejectsDuplicateMissingAndRepeatedChildIdentifiers() {
    reject(
      packet([
        ["id": 1, "kind": "view", "children": []],
        ["id": 1, "kind": "text", "children": [], "text": "x"],
      ]), code: "duplicate-node")
    reject(packet([["id": 1, "kind": "view", "children": [2]]]), code: "missing-child")
    reject(
      packet([
        ["id": 1, "kind": "view", "children": [2, 2]],
        ["id": 2, "kind": "text", "children": [], "text": "x"],
      ]), code: "duplicate-child")
    reject(packet([["id": 2, "kind": "view", "children": []]]), code: "missing-root")
  }

  func testRejectsCyclesSharedChildrenAndUnreachableNodes() {
    reject(
      packet([
        ["id": 1, "kind": "view", "children": [2]], ["id": 2, "kind": "view", "children": [1]],
      ]), code: "cyclic-tree")
    reject(
      packet([
        ["id": 1, "kind": "view", "children": [2, 3]],
        ["id": 2, "kind": "view", "children": [4]],
        ["id": 3, "kind": "view", "children": [4]],
        ["id": 4, "kind": "text", "children": [], "text": "x"],
      ]), code: "multiple-parents")
    reject(
      packet([
        ["id": 1, "kind": "view", "children": []], ["id": 2, "kind": "view", "children": []],
      ]), code: "unreachable-node")
    reject(
      packet([
        ["id": 1, "kind": "view", "children": []], ["id": 2, "kind": "view", "children": [1]],
      ]), code: "invalid-root")
  }

  func testRejectsUnsupportedKindsPropertiesAndLeafChildren() {
    reject(packet([["id": 1, "kind": "image", "children": []]]), code: "unsupported-kind")
    reject(
      packet([["id": 1, "kind": "view", "children": [], "color": "red"]]),
      code: "unsupported-property")
    reject(
      packet([["id": 1, "kind": "text", "children": [], "text": "x", "gap": 0]]),
      code: "unsupported-property")
    reject(
      packet([["id": 1, "kind": "button", "children": [2], "label": "x"]]),
      code: "unsupported-children")
    var value = counter()
    value["executor"] = "unexpected"
    reject(value, code: "unsupported-property")
  }

  func testNumericAdmissionNeverConfusesBooleansWithIdentifiers() {
    for value: Any in [
      true, false, 0, -1, 1.5, Double.nan, Double.infinity, 9_007_199_254_740_992.0,
    ] {
      var input = counter()
      input["root"] = value
      reject(input, code: "invalid-integer")
    }
    var input = counter()
    input["formatVersion"] = true
    reject(input, code: "invalid-integer")
    input["formatVersion"] = 2
    reject(input, code: "unsupported-version")
  }

  func testLayoutValuesAreFiniteAndBounded() throws {
    for value: Any in [true, -1, 4_097, Double.nan, Double.infinity] {
      reject(
        packet([["id": 1, "kind": "view", "children": [], "gap": value]]),
        code: "invalid-layout-value")
    }
    for value in [0.0, 4_096.0] {
      guard
        case .render(let result) = try NativeBridgeParser.parse(
          packet([
            ["id": 1, "kind": "view", "children": [], "gap": value, "padding": value]
          ]))
      else { return XCTFail("expected a bounded layout") }
      XCTAssertEqual(result.nodes[1]?.gap, value)
    }
  }

  func testEmptyTextIsValidButAnUnnamedButtonIsNot() throws {
    _ = try NativeBridgeParser.parse(
      packet([["id": 1, "kind": "text", "children": [], "text": ""]]))
    reject(
      packet([["id": 1, "kind": "button", "children": [], "label": " \n"]]), code: "invalid-string")
    reject(
      packet([
        ["id": 1, "kind": "text", "children": [], "text": String(repeating: "x", count: 4_097)]
      ]), code: "invalid-string")
  }

  func testGraphBudgetsBoundAdmissionBeforeNativeConstruction() throws {
    func chain(_ count: Int) -> [[String: Any]] {
      (1...count).map { id in
        id == count
          ? ["id": id, "kind": "text", "children": [], "text": "end"]
          : ["id": id, "kind": "view", "children": [id + 1]]
      }
    }
    _ = try NativeBridgeParser.parse(packet(chain(64)))
    reject(packet(chain(65)), code: "maximum-depth")
    reject(packet(chain(1_025)), code: "invalid-nodes")
  }
}
