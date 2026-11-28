import XCTest

/// These queries inspect UIKit elements; the execution web view is hidden from AX.
final class NativeHostInteractionTests: XCTestCase {
  override func setUpWithError() throws { continueAfterFailure = false }

  @MainActor private func launch(_ scenario: String) -> XCUIApplication {
    let app = XCUIApplication()
    app.launchEnvironment["VIZE_NATIVE_SCENARIO"] = scenario
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

  @MainActor private func retainScreenshot(_ app: XCUIApplication, name: String) {
    let attachment = XCTAttachment(screenshot: app.screenshot())
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)
  }
}
