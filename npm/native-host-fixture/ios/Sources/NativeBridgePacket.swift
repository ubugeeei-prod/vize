import CoreFoundation
import Foundation

/// The private first host accepts only version-one view trees and diagnostics.
enum NativeBridgeMessage {
  case render(NativeViewPacket)
  case diagnostic(NativeDiagnostic)
}

/// A complete, validated snapshot. Node identity is stable within this packet.
struct NativeViewPacket {
  let sequence: Int
  let root: Int
  let nodes: [Int: NativeViewNode]
}

/// Supported primitives deliberately exclude images, animation, and arbitrary CSS.
struct NativeViewNode {
  enum Kind: String {
    case view
    case text
    case button
  }

  let id: Int
  let kind: Kind
  let children: [Int]
  /// Required text content for text primitives; other kinds store an empty string.
  let text: String
  let label: String
  /// Vertical stack spacing. Defaults to zero; supported range is 0...4096.
  let gap: Double
  /// Equal directional stack insets. Defaults to zero; supported range is 0...4096.
  let padding: Double
}

struct NativeDiagnostic {
  let code: String
  let message: String
}

/// Stable failures are rendered by native UIKit, including malformed JS packets.
struct NativeBridgeFailure: Error {
  let code: String
  let message: String
}

/// Pure admission boundary used before any UIKit view is created or replaced.
///
/// IDs and sequences use positive JavaScript-safe integers. Trees are bounded to
/// 1,024 nodes, depth 64, and 4,096 UTF-16 units per string. Only view primitives
/// accept gap/padding; leaf primitives require empty children. All nodes must be
/// reachable exactly once from the selected root. Parsing never mutates inputs.
enum NativeBridgeParser {
  private static let maximumInteger = 9_007_199_254_740_991.0
  private static let maximumNodes = 1_024
  private static let maximumDepth = 64

  /// The initial accepted sequence defaults to zero; later snapshots must advance it.
  static func parse(_ body: Any, after acceptedSequence: Int = 0) throws -> NativeBridgeMessage {
    let object = try record(body, path: "message")
    guard let format = object["format"] as? String else {
      throw failure("invalid-format", "message.format must be a supported string")
    }
    switch format {
    case "vize.native-view-tree":
      try fields(
        object, allowed: ["format", "formatVersion", "sequence", "root", "nodes"], path: "message")
      try version(object)
      return .render(try render(object, after: acceptedSequence))
    case "vize.native-diagnostic":
      try fields(object, allowed: ["format", "formatVersion", "code", "message"], path: "message")
      try version(object)
      return .diagnostic(
        NativeDiagnostic(
          code: try string(object["code"], path: "message.code", maximum: 128, allowEmpty: false),
          message: try string(object["message"], path: "message.message", allowEmpty: false)
        ))
    default:
      throw failure("unsupported-format", "message.format is not supported by this host")
    }
  }

  private static func render(_ object: [String: Any], after acceptedSequence: Int) throws
    -> NativeViewPacket
  {
    let sequence = try integer(object["sequence"], path: "message.sequence")
    guard sequence > acceptedSequence else {
      throw failure("stale-sequence", "message.sequence must advance the accepted snapshot")
    }
    let root = try integer(object["root"], path: "message.root")
    guard let values = object["nodes"] as? [Any], !values.isEmpty, values.count <= maximumNodes
    else {
      throw failure("invalid-nodes", "message.nodes must contain 1...1024 nodes")
    }
    var nodes: [Int: NativeViewNode] = [:]
    for (index, value) in values.enumerated() {
      let path = "nodes.\(index)"
      let source = try record(value, path: path)
      guard let name = source["kind"] as? String, let kind = NativeViewNode.Kind(rawValue: name)
      else {
        throw failure("unsupported-kind", "\(path).kind is not supported by this host")
      }
      let extra: Set<String>
      switch kind {
      case .view: extra = ["gap", "padding"]
      case .text: extra = ["text"]
      case .button: extra = ["label"]
      }
      try fields(source, allowed: Set(["id", "kind", "children"]).union(extra), path: path)
      let id = try integer(source["id"], path: "\(path).id")
      guard nodes[id] == nil else {
        throw failure("duplicate-node", "node identifiers must be unique")
      }
      guard let rawChildren = source["children"] as? [Any], rawChildren.count <= maximumNodes else {
        throw failure("invalid-children", "\(path).children must be an array of node identifiers")
      }
      let children = try rawChildren.enumerated().map {
        try integer($0.element, path: "\(path).children.\($0.offset)")
      }
      guard Set(children).count == children.count else {
        throw failure("duplicate-child", "\(path).children must not repeat an identifier")
      }
      guard kind == .view || children.isEmpty else {
        throw failure("unsupported-children", "text and button primitives cannot own children")
      }
      nodes[id] = NativeViewNode(
        id: id, kind: kind, children: children,
        text: kind == .text ? try string(source["text"], path: "\(path).text") : "",
        label: kind == .button
          ? try string(source["label"], path: "\(path).label", allowEmpty: false) : "",
        gap: kind == .view ? try dimension(source["gap"], path: "\(path).gap") : 0,
        padding: kind == .view ? try dimension(source["padding"], path: "\(path).padding") : 0
      )
    }
    try graph(nodes, root: root)
    return NativeViewPacket(sequence: sequence, root: root, nodes: nodes)
  }

  private static func graph(_ nodes: [Int: NativeViewNode], root: Int) throws {
    guard nodes[root] != nil else {
      throw failure("missing-root", "message.root must name a declared node")
    }
    let identifiers = nodes.keys.sorted()
    var parents: [Int: Int] = [:]
    for id in identifiers {
      for child in nodes[id]!.children {
        guard nodes[child] != nil else {
          throw failure("missing-child", "node \(id) names an undeclared child")
        }
        guard parents[child] == nil else {
          throw failure("multiple-parents", "each child must have exactly one parent")
        }
        parents[child] = id
      }
    }
    // Iteration bounds the native call stack even for an adversarial cyclic packet.
    var colors: [Int: Int] = [:]
    for start in identifiers where colors[start] == nil {
      var stack: [(id: Int, next: Int)] = [(start, 0)]
      colors[start] = 1
      while let frame = stack.last {
        let children = nodes[frame.id]!.children
        if frame.next == children.count {
          colors[frame.id] = 2
          stack.removeLast()
          continue
        }
        let child = children[frame.next]
        stack[stack.count - 1].next += 1
        if colors[child] == 1 {
          throw failure("cyclic-tree", "native view trees must not contain a cycle")
        }
        if colors[child] == nil {
          colors[child] = 1
          stack.append((child, 0))
        }
      }
    }
    guard parents[root] == nil else {
      throw failure("invalid-root", "the selected root cannot have a parent")
    }
    var seen: Set<Int> = []
    var pending: [(id: Int, depth: Int)] = [(root, 1)]
    while let entry = pending.popLast() {
      guard entry.depth <= maximumDepth else {
        throw failure("maximum-depth", "native view trees cannot exceed depth 64")
      }
      seen.insert(entry.id)
      for child in nodes[entry.id]!.children { pending.append((child, entry.depth + 1)) }
    }
    guard seen.count == nodes.count else {
      throw failure("unreachable-node", "every node must be reachable from message.root")
    }
  }

  private static func version(_ object: [String: Any]) throws {
    guard try integer(object["formatVersion"], path: "message.formatVersion") == 1 else {
      throw failure("unsupported-version", "message.formatVersion must equal 1")
    }
  }

  private static func record(_ value: Any, path: String) throws -> [String: Any] {
    guard let object = value as? [String: Any] else {
      throw failure("invalid-object", "\(path) must be an object")
    }
    return object
  }

  private static func fields(_ object: [String: Any], allowed: Set<String>, path: String) throws {
    if let unknown = object.keys.filter({ !allowed.contains($0) }).sorted().first {
      throw failure("unsupported-property", "\(path).\(unknown) is not supported by this host")
    }
  }

  private static func integer(_ value: Any?, path: String) throws -> Int {
    guard let number = value as? NSNumber, CFGetTypeID(number) != CFBooleanGetTypeID() else {
      throw failure("invalid-integer", "\(path) must be a positive safe integer")
    }
    let raw = number.doubleValue
    guard raw.isFinite, raw >= 1, raw <= maximumInteger, raw.rounded(.towardZero) == raw else {
      throw failure("invalid-integer", "\(path) must be a positive safe integer")
    }
    return Int(number.int64Value)
  }

  private static func dimension(_ value: Any?, path: String) throws -> Double {
    guard let value = value else { return 0 }
    guard let number = value as? NSNumber, CFGetTypeID(number) != CFBooleanGetTypeID(),
      number.doubleValue.isFinite, (0...4096).contains(number.doubleValue)
    else {
      throw failure("invalid-layout-value", "\(path) must be a finite number in 0...4096")
    }
    return number.doubleValue
  }

  private static func string(
    _ value: Any?, path: String, maximum: Int = 4_096, allowEmpty: Bool = true
  ) throws -> String {
    guard let text = value as? String, text.utf16.count <= maximum,
      allowEmpty || !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    else {
      throw failure("invalid-string", "\(path) must be a supported string")
    }
    return text
  }

  private static func failure(_ code: String, _ message: String) -> NativeBridgeFailure {
    NativeBridgeFailure(code: code, message: message)
  }
}
