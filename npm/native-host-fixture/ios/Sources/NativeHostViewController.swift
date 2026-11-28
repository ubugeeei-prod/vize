import UIKit
import WebKit

/// A deliberately small engine adapter: JS supplies snapshots; UIKit owns visible UI.
///
/// This host provides vertical stack layout, native text, accessible buttons, and
/// press delivery only. The hidden web view executes application JavaScript;
/// it cannot satisfy native accessibility assertions in the UI test target.
final class NativeHostViewController: UIViewController, WKNavigationDelegate {
  private let content = UIStackView()
  private let diagnostic = UILabel()
  private var engine: WKWebView!
  private var location: NativeHostLocation?
  private var developmentDocumentCommitted = false
  private var refusedNavigation = false
  private var acceptedSequence = 0
  private var currentButtons: Set<Int> = []

  override func viewDidLoad() {
    super.viewDidLoad()
    view.backgroundColor = .systemBackground
    configureNativeContainer()
    configureEngine()
  }

  private func configureNativeContainer() {
    diagnostic.accessibilityIdentifier = "native-diagnostic"
    diagnostic.font = .preferredFont(forTextStyle: .body)
    diagnostic.adjustsFontForContentSizeCategory = true
    diagnostic.numberOfLines = 0
    diagnostic.textColor = .systemRed
    diagnostic.isAccessibilityElement = true
    diagnostic.isHidden = true

    content.axis = .vertical
    content.alignment = .fill
    content.accessibilityIdentifier = "native-root"
    let chrome = UIStackView(arrangedSubviews: [diagnostic, content])
    chrome.axis = .vertical
    chrome.spacing = 12
    chrome.translatesAutoresizingMaskIntoConstraints = false
    view.addSubview(chrome)
    NSLayoutConstraint.activate([
      chrome.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
      chrome.leadingAnchor.constraint(
        equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 16),
      chrome.trailingAnchor.constraint(
        equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -16),
      chrome.bottomAnchor.constraint(
        lessThanOrEqualTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -16),
    ])
  }

  private func configureEngine() {
    let controller = WKUserContentController()
    controller.add(WeakNativeMessageHandler(owner: self), name: "vizeNative")
    let scenario = ProcessInfo.processInfo.environment["VIZE_NATIVE_SCENARIO"] ?? "counter"
    do {
      // Values are serialized as JSON; no environment value is treated as source.
      let boot = try JSONSerialization.data(
        withJSONObject: ["scenario": scenario], options: [.sortedKeys])
      let source =
        "globalThis.__VIZE_NATIVE_BOOT__ = " + String(decoding: boot, as: UTF8.self) + ";"
      controller.addUserScript(
        WKUserScript(source: source, injectionTime: .atDocumentStart, forMainFrameOnly: true))
    } catch {
      show(
        NativeDiagnostic(
          code: "invalid-boot-config", message: "native boot configuration could not be encoded"))
      return
    }
    let configuration = WKWebViewConfiguration()
    configuration.userContentController = controller
    configuration.websiteDataStore = .nonPersistent()
    let engine = WKWebView(
      frame: CGRect(x: 0, y: 0, width: 1, height: 1), configuration: configuration)
    engine.navigationDelegate = self
    engine.isHidden = true
    engine.accessibilityElementsHidden = true
    engine.isAccessibilityElement = false
    view.addSubview(engine)
    self.engine = engine

    do {
      let selected = try NativeHostLocation.forCurrentBuild(
        environment: ProcessInfo.processInfo.environment,
        bundledIndex: Bundle.main.url(forResource: "index", withExtension: "html"))
      location = selected
      switch selected {
      case .bundled(let index):
        guard Bundle.main.url(forResource: "application", withExtension: "js") != nil else {
          throw NativeBridgeFailure(
            code: "missing-resource", message: "the offline application bundle is missing")
        }
        engine.loadFileURL(index, allowingReadAccessTo: index.deletingLastPathComponent())
      case .development(let root):
        engine.load(URLRequest(url: root))
      }
    } catch let failure as NativeBridgeFailure {
      show(NativeDiagnostic(code: failure.code, message: failure.message))
    } catch {
      show(
        NativeDiagnostic(
          code: "invalid-host-location", message: "The application location could not be selected.")
      )
    }
  }

  fileprivate func receive(_ message: WKScriptMessage) {
    guard message.name == "vizeNative", message.frameInfo.isMainFrame,
      message.webView === engine, location?.permitsMainFrame(message.frameInfo.request.url) == true
    else {
      show(
        NativeDiagnostic(
          code: "untrusted-message",
          message: "native messages must originate in the selected application main frame"))
      return
    }
    do {
      switch try NativeBridgeParser.parse(message.body, after: acceptedSequence) {
      case .diagnostic(let value): show(value)
      case .render(let packet): install(packet)
      }
    } catch let error as NativeBridgeFailure {
      show(NativeDiagnostic(code: error.code, message: error.message))
    } catch {
      show(
        NativeDiagnostic(
          code: "invalid-message", message: "the native message could not be admitted"))
    }
  }

  private func install(_ packet: NativeViewPacket) {
    // The entire graph is validated before a replacement tree is constructed.
    let replacement = makeView(packet.root, packet: packet)
    for child in content.arrangedSubviews {
      content.removeArrangedSubview(child)
      child.removeFromSuperview()
    }
    acceptedSequence = packet.sequence
    currentButtons = Set(packet.nodes.values.filter { $0.kind == .button }.map(\.id))
    content.addArrangedSubview(replacement)
    content.isUserInteractionEnabled = true
    diagnostic.isHidden = true
    diagnostic.text = nil
  }

  private func makeView(_ id: Int, packet: NativeViewPacket) -> UIView {
    // Admission proves every referenced ID exists and bounds recursion to depth 64.
    let node = packet.nodes[id]!
    let result: UIView
    switch node.kind {
    case .view:
      let stack = UIStackView()
      stack.axis = .vertical
      stack.alignment = .fill
      stack.spacing = CGFloat(node.gap)
      stack.isLayoutMarginsRelativeArrangement = true
      let inset = CGFloat(node.padding)
      stack.directionalLayoutMargins = NSDirectionalEdgeInsets(
        top: inset, leading: inset, bottom: inset, trailing: inset)
      for child in node.children { stack.addArrangedSubview(makeView(child, packet: packet)) }
      result = stack
    case .text:
      let label = UILabel()
      label.text = node.text
      label.numberOfLines = 0
      label.font = .preferredFont(forTextStyle: .body)
      label.adjustsFontForContentSizeCategory = true
      label.isAccessibilityElement = true
      result = label
    case .button:
      let button = UIButton(type: .system)
      button.setTitle(node.label, for: .normal)
      button.titleLabel?.font = .preferredFont(forTextStyle: .body)
      button.titleLabel?.adjustsFontForContentSizeCategory = true
      button.heightAnchor.constraint(greaterThanOrEqualToConstant: 44).isActive = true
      button.addAction(
        UIAction { [weak self] _ in
          self?.press(node: id, sourceSequence: packet.sequence)
        }, for: .touchUpInside)
      result = button
    }
    result.accessibilityIdentifier = "native-node-\(id)"
    result.translatesAutoresizingMaskIntoConstraints = false
    return result
  }

  private func press(node: Int, sourceSequence: Int) {
    guard sourceSequence == acceptedSequence, currentButtons.contains(node) else { return }
    let event: [String: Any] = ["node": node, "event": "press", "sequence": sourceSequence]
    // WebKit encodes the argument object. Untrusted values never become JS source.
    engine.callAsyncJavaScript(
      "await globalThis.__VIZE_NATIVE__.dispatch(event); return true;",
      arguments: ["event": event], in: nil, in: .page
    ) { [weak self] result in
      if case .failure = result {
        self?.show(
          NativeDiagnostic(
            code: "event-dispatch-failed", message: "the application event dispatcher failed"))
      }
    }
  }

  private func show(_ value: NativeDiagnostic) {
    currentButtons = []
    content.isUserInteractionEnabled = false
    diagnostic.text = "\(value.code): \(value.message)"
    diagnostic.isHidden = false
  }

  func webView(
    _ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction,
    decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
  ) {
    guard let location = location else {
      decisionHandler(.cancel)
      return
    }
    if let failure = location.navigationRefusal(
      url: navigationAction.request.url,
      isMainFrame: navigationAction.targetFrame?.isMainFrame == true,
      documentAlreadyLoaded: developmentDocumentCommitted || acceptedSequence > 0
    ) {
      decisionHandler(.cancel)
      refusedNavigation = true
      show(NativeDiagnostic(code: failure.code, message: failure.message))
      return
    }
    refusedNavigation = false
    decisionHandler(.allow)
  }

  func webView(
    _ webView: WKWebView, decidePolicyFor navigationResponse: WKNavigationResponse,
    decisionHandler: @escaping (WKNavigationResponsePolicy) -> Void
  ) {
    // A redirected response must pass the same bound-document check before commit.
    guard navigationResponse.isForMainFrame,
      location?.permitsMainFrame(navigationResponse.response.url) == true
    else {
      decisionHandler(.cancel)
      refusedNavigation = true
      show(
        NativeDiagnostic(
          code: "native-navigation-refused",
          message: "Navigation is restricted to the selected application document."))
      return
    }
    decisionHandler(.allow)
  }

  func webView(_ webView: WKWebView, didCommit navigation: WKNavigation!) {
    if location?.isDevelopment == true, location?.permitsMainFrame(webView.url) == true {
      developmentDocumentCommitted = true
    }
  }

  func webView(
    _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
    withError error: Error
  ) {
    if refusedNavigation { return }
    show(
      NativeDiagnostic(
        code: "engine-load-failed", message: "the selected application could not be loaded"))
  }

  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
    if refusedNavigation { return }
    show(
      NativeDiagnostic(
        code: "engine-load-failed", message: "the selected application could not be loaded"))
  }

  func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
    currentButtons = []
    content.isUserInteractionEnabled = false
    show(
      NativeDiagnostic(
        code: "engine-terminated", message: "the application execution process terminated"))
  }
}

/// Breaks the user-content-controller retention cycle without changing message scope.
private final class WeakNativeMessageHandler: NSObject, WKScriptMessageHandler {
  private weak var owner: NativeHostViewController?
  init(owner: NativeHostViewController) { self.owner = owner }
  func userContentController(
    _ userContentController: WKUserContentController, didReceive message: WKScriptMessage
  ) {
    owner?.receive(message)
  }
}
