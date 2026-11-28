import Foundation

/// The selected document owns the native bridge for the lifetime of this host.
///
/// Offline resources remain the default. Development mode is explicitly opted
/// into with VIZE_NATIVE_DEV_URL and is compiled out of Release admission. The
/// only supported network root is a canonical IPv4 loopback HTTP URL with an
/// explicit port. Credentials, aliases, extra paths, queries, and fragments are
/// refused before Foundation or the browser can normalize them away.
enum NativeHostLocation {
  case bundled(URL)
  case development(URL)

  var url: URL {
    switch self {
    case .bundled(let url), .development(let url): return url
    }
  }

  var isDevelopment: Bool {
    if case .development = self { return true }
    return false
  }

  /// Build configuration, rather than an environment variable, enables networking.
  static func forCurrentBuild(environment: [String: String], bundledIndex: URL?) throws
    -> NativeHostLocation
  {
    #if DEBUG
      let developmentEnabled = true
    #else
      let developmentEnabled = false
    #endif
    return try select(
      developmentURL: environment["VIZE_NATIVE_DEV_URL"], bundledIndex: bundledIndex,
      developmentEnabled: developmentEnabled)
  }

  /// Pure admission seam used to exercise both Debug and Release policies.
  static func select(developmentURL: String?, bundledIndex: URL?, developmentEnabled: Bool) throws
    -> NativeHostLocation
  {
    if let value = developmentURL {
      guard developmentEnabled else {
        throw NativeBridgeFailure(
          code: "native-development-unavailable",
          message: "Development URLs are supported only by Debug builds.")
      }
      return .development(try loopbackRoot(value))
    }
    guard let index = bundledIndex, index.isFileURL else {
      throw NativeBridgeFailure(
        code: "missing-resource", message: "the offline application bundle is missing")
    }
    return .bundled(index.standardizedFileURL)
  }

  /// Shared admission for navigation responses and messages from the main frame.
  func permitsMainFrame(_ candidate: URL?) -> Bool {
    guard let candidate = candidate,
      candidate.user == nil, candidate.password == nil, candidate.query == nil,
      candidate.fragment == nil
    else { return false }
    switch self {
    case .bundled(let selected):
      return candidate.isFileURL && (candidate.host == nil || candidate.host == "")
        && candidate.standardizedFileURL.path == selected.path
    case .development(let selected):
      // Browsers may elide the default HTTP port while retaining the same origin.
      return candidate.scheme == "http" && candidate.host == "127.0.0.1"
        && (candidate.port ?? 80) == (selected.port ?? 80)
        && candidate.path == "/"
        && URLComponents(url: candidate, resolvingAgainstBaseURL: false)?.percentEncodedPath == "/"
    }
  }

  /// Template HMR keeps one document. Reload needs a future document-epoch protocol.
  func navigationRefusal(url: URL?, isMainFrame: Bool, documentAlreadyLoaded: Bool)
    -> NativeBridgeFailure?
  {
    guard isMainFrame && permitsMainFrame(url) else {
      return NativeBridgeFailure(
        code: "native-navigation-refused",
        message: "Navigation is restricted to the selected application document.")
    }
    if isDevelopment && documentAlreadyLoaded {
      return NativeBridgeFailure(
        code: "native-development-reload-unavailable",
        message:
          "Document reload requires restarting this development client; template refresh remains available."
      )
    }
    return nil
  }

  private static func loopbackRoot(_ value: String) throws -> URL {
    let prefix = "http://127.0.0.1:"
    guard value.hasPrefix(prefix), value.hasSuffix("/") else { throw invalidDevelopmentURL() }
    let digits = value.dropFirst(prefix.count).dropLast()
    guard (1...5).contains(digits.utf8.count),
      digits.utf8.allSatisfy({ (48...57).contains($0) }),
      let port = Int(digits), (1...65535).contains(port), String(port) == String(digits),
      let url = URL(string: value)
    else { throw invalidDevelopmentURL() }
    return url
  }

  private static func invalidDevelopmentURL() -> NativeBridgeFailure {
    NativeBridgeFailure(
      code: "invalid-development-url",
      message: "Development requires exactly http://127.0.0.1:<port>/ with port 1...65535.")
  }
}
