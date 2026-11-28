import Foundation
import XCTest

@testable import NativePacket

final class NativeHostLocationTests: XCTestCase {
  private let bundled = URL(fileURLWithPath: "/tmp/vize-native-location-fixture/index.html")

  private func development(_ value: String = "http://127.0.0.1:5173/") throws -> NativeHostLocation
  {
    try NativeHostLocation.select(
      developmentURL: value, bundledIndex: bundled, developmentEnabled: true)
  }

  private func reject(
    _ value: String, code: String = "invalid-development-url", enabled: Bool = true,
    file: StaticString = #filePath, line: UInt = #line
  ) {
    do {
      _ = try NativeHostLocation.select(
        developmentURL: value, bundledIndex: bundled, developmentEnabled: enabled)
      XCTFail("expected refusal \(code)", file: file, line: line)
    } catch let failure as NativeBridgeFailure {
      XCTAssertEqual(failure.code, code, file: file, line: line)
    } catch { XCTFail("expected a stable location refusal", file: file, line: line) }
  }

  func testAcceptsOnlyCanonicalLoopbackPorts() throws {
    for port in [1, 80, 5173, 65535] {
      let value = "http://127.0.0.1:\(port)/"
      let location = try development(value)
      XCTAssertTrue(location.isDevelopment)
      XCTAssertEqual(location.url.absoluteString, value)
      XCTAssertTrue(location.permitsMainFrame(location.url))
    }
  }

  func testRejectsAliasesCredentialsAndAllExtraURLComponents() {
    for value in [
      "", "http://127.0.0.1/", "http://127.0.0.1:0/", "http://127.0.0.1:65536/",
      "http://127.0.0.1:080/", "http://127.0.0.1:0001/", "http://127.0.0.1:+1/",
      "http://127.0.0.1:1.0/", "http://127.0.0.1:١/", "http://127.0.0.1:5173",
      "https://127.0.0.1:5173/", "HTTP://127.0.0.1:5173/", "//127.0.0.1:5173/",
      "http://localhost:5173/", "http://[::1]:5173/", "http://127.0.0.2:5173/",
      "http://127.1:5173/", "http://2130706433:5173/", "http://0x7f000001:5173/",
      "http://127.0.0.1.:5173/", "http://127.000.000.001:5173/",
      "http://user@127.0.0.1:5173/", "http://@127.0.0.1:5173/",
      "http://user:secret@127.0.0.1:5173/", "http://127.0.0.1:5173//",
      "http://127.0.0.1:5173/./", "http://127.0.0.1:5173/%2e/",
      "http://127.0.0.1:5173/?", "http://127.0.0.1:5173/?query=1",
      "http://127.0.0.1:5173/#", "http://127.0.0.1:5173/#fragment",
      " http://127.0.0.1:5173/", "http://127.0.0.1:5173/\n",
    ] { reject(value) }
  }

  func testReleasePolicyRefusesEveryConfiguredDevelopmentOption() {
    for value in ["", "http://127.0.0.1:5173/", "https://example.invalid/"] {
      reject(value, code: "native-development-unavailable", enabled: false)
    }
  }

  func testTheCurrentBuildUsesItsCompileTimeDevelopmentGate() throws {
    let environment = ["VIZE_NATIVE_DEV_URL": "http://127.0.0.1:5173/"]
    #if DEBUG
      XCTAssertTrue(
        try NativeHostLocation.forCurrentBuild(environment: environment, bundledIndex: bundled)
          .isDevelopment)
    #else
      XCTAssertThrowsError(
        try NativeHostLocation.forCurrentBuild(environment: environment, bundledIndex: bundled)
      ) {
        XCTAssertEqual(($0 as? NativeBridgeFailure)?.code, "native-development-unavailable")
      }
    #endif
  }

  func testTheOfflineDefaultRemainsBoundToTheBundledDocument() throws {
    let location = try NativeHostLocation.forCurrentBuild(environment: [:], bundledIndex: bundled)
    XCTAssertFalse(location.isDevelopment)
    XCTAssertEqual(location.url, bundled)
    XCTAssertTrue(location.permitsMainFrame(bundled))
    XCTAssertFalse(location.permitsMainFrame(URL(fileURLWithPath: "/tmp/other/index.html")))
    XCTAssertFalse(location.permitsMainFrame(URL(string: "http://127.0.0.1:5173/")))
    XCTAssertFalse(location.permitsMainFrame(URL(string: bundled.absoluteString + "?query=1")))
    XCTAssertFalse(location.permitsMainFrame(URL(string: bundled.absoluteString + "#fragment")))
  }

  func testNavigationAndMessagesCannotLeaveTheSelectedRootOrigin() throws {
    let location = try development()
    for value in [
      "http://127.0.0.1:5174/", "https://127.0.0.1:5173/", "http://localhost:5173/",
      "http://127.0.0.2:5173/", "http://127.0.0.1:5173/other", "http://127.0.0.1:5173//",
      "http://127.0.0.1:5173/?", "http://127.0.0.1:5173/#",
      "http://user@127.0.0.1:5173/", "file:///tmp/index.html",
    ] { XCTAssertFalse(location.permitsMainFrame(URL(string: value)), value) }
    XCTAssertFalse(location.permitsMainFrame(nil))
    XCTAssertEqual(
      location.navigationRefusal(
        url: location.url, isMainFrame: false,
        documentAlreadyLoaded: false)?.code, "native-navigation-refused")
    // An HTTP browser may canonicalize an explicit default port out of its URL.
    XCTAssertTrue(
      try development("http://127.0.0.1:80/").permitsMainFrame(URL(string: "http://127.0.0.1/")))
  }

  func testDocumentReloadRequiresAnExplicitFutureEpochProtocol() throws {
    let location = try development()
    XCTAssertNil(
      location.navigationRefusal(url: location.url, isMainFrame: true, documentAlreadyLoaded: false)
    )
    XCTAssertEqual(
      location.navigationRefusal(
        url: location.url, isMainFrame: true,
        documentAlreadyLoaded: true)?.code, "native-development-reload-unavailable")
    XCTAssertTrue(
      location.permitsMainFrame(location.url),
      "same-document template updates retain bridge admission")
  }

  func testOnlyExplicitDevelopmentModeCanStartWithoutBundledResources() throws {
    XCTAssertThrowsError(
      try NativeHostLocation.select(
        developmentURL: nil, bundledIndex: nil, developmentEnabled: true)
    ) {
      XCTAssertEqual(($0 as? NativeBridgeFailure)?.code, "missing-resource")
    }
    XCTAssertThrowsError(
      try NativeHostLocation.select(
        developmentURL: nil,
        bundledIndex: URL(string: "http://127.0.0.1:5173/"), developmentEnabled: true)
    ) {
      XCTAssertEqual(($0 as? NativeBridgeFailure)?.code, "missing-resource")
    }
    XCTAssertTrue(
      try NativeHostLocation.select(
        developmentURL: "http://127.0.0.1:5173/",
        bundledIndex: nil, developmentEnabled: true
      ).isDevelopment)
  }
}
