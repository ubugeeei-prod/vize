import XCTest

/// These queries inspect UIKit elements; the execution web view is hidden from AX.
final class NativeHostInteractionTests: XCTestCase {
  override func setUpWithError() throws { continueAfterFailure = false }

  @MainActor private func launch(_ scenario: String, developmentURL: String? = nil)
    -> XCUIApplication
  {
    let app = XCUIApplication()
    app.launchEnvironment["VIZE_NATIVE_SCENARIO"] = scenario
    if let developmentURL { app.launchEnvironment["VIZE_NATIVE_DEV_URL"] = developmentURL }
    app.launchArguments = ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
    app.launch()
    return app
  }

  @MainActor func testNativeCounterPressReachesTheSFCApplication() {
    let app = launch("counter")
    defer { app.terminate() }
    let initial = app.staticTexts["Count: 0"]
    XCTAssertTrue(initial.waitForExistence(timeout: 20))
    XCTAssertTrue(initial.identifier.hasPrefix("native-node-"))
    XCTAssertFalse(app.webViews.firstMatch.exists)
    retainScreenshot(app, name: "native-counter-initial")
    let increment = app.buttons["Increment"]
    XCTAssertTrue(increment.waitForExistence(timeout: 5))
    XCTAssertTrue(increment.identifier.hasPrefix("native-node-"))
    increment.tap()
    XCTAssertTrue(app.staticTexts["Count: 1"].waitForExistence(timeout: 10))
    XCTAssertFalse(app.staticTexts["native-diagnostic"].exists)
    retainScreenshot(app, name: "native-counter-after-press")
  }

  @MainActor func testUnsupportedCapabilityShowsANativeDiagnosticBeforeMounting() {
    let app = launch("unsupported-capability")
    defer { app.terminate() }
    let diagnostic = app.staticTexts["native-diagnostic"]
    XCTAssertTrue(diagnostic.waitForExistence(timeout: 20))
    XCTAssertTrue(diagnostic.label.contains("missing-capability"))
    XCTAssertTrue(diagnostic.label.contains("native.images"))
    XCTAssertFalse(app.staticTexts["Count: 0"].exists)
    XCTAssertFalse(app.buttons["Increment"].exists)
    XCTAssertFalse(app.webViews.firstMatch.exists)
    retainScreenshot(app, name: "native-unsupported-capability")
  }

  @MainActor func testNativeTemplateRefreshPreservesSetupState() {
    guard let origin = ProcessInfo.processInfo.environment["VIZE_NATIVE_REFRESH_ORIGIN"] else {
      XCTFail("Native refresh qualification requires its invocation-owned development origin.")
      return
    }
    let app = launch("counter", developmentURL: origin)
    defer { app.terminate() }
    let initial = app.staticTexts["Count: 0"]
    XCTAssertTrue(initial.waitForExistence(timeout: 20))
    let identifier = initial.identifier
    XCTAssertTrue(identifier.hasPrefix("native-node-"))
    let increment = app.buttons["Increment"]
    increment.tap()
    XCTAssertTrue(app.staticTexts["Count: 1"].waitForExistence(timeout: 10))
    FileHandle.standardOutput.write(Data("VIZE_NATIVE_TEMPLATE_REFRESH_READY\n".utf8))
    let refreshed = app.staticTexts["Refreshed count: 1"]
    XCTAssertTrue(refreshed.waitForExistence(timeout: 40))
    XCTAssertEqual(refreshed.identifier, identifier)
    app.buttons["Increment"].tap()
    XCTAssertTrue(app.staticTexts["Refreshed count: 2"].waitForExistence(timeout: 10))
    XCTAssertFalse(app.staticTexts["native-diagnostic"].exists)
    XCTAssertFalse(app.webViews.firstMatch.exists)
    retainScreenshot(app, name: "native-template-refresh-preserved-state")
  }

  #if !DEBUG
    @MainActor func testReleaseDevelopmentOptionIsRefused() {
      let app = launch("counter", developmentURL: "http://127.0.0.1:1/")
      defer { app.terminate() }
      let diagnostic = app.staticTexts["native-diagnostic"]
      XCTAssertTrue(diagnostic.waitForExistence(timeout: 20))
      XCTAssertTrue(diagnostic.label.contains("native-development-unavailable"))
      XCTAssertFalse(app.staticTexts["Count: 0"].exists)
      XCTAssertFalse(app.buttons["Increment"].exists)
      XCTAssertFalse(app.webViews.firstMatch.exists)
      retainScreenshot(app, name: "native-release-development-refused")
    }
  #endif

  @MainActor private func retainScreenshot(_ app: XCUIApplication, name: String) {
    let attachment = XCTAttachment(screenshot: app.screenshot())
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
  }
}
