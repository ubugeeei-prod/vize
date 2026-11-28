// swift-tools-version: 5.9
import PackageDescription

// Pure packet admission can be tested on macOS without starting UIKit or WebKit.
let package = Package(
  name: "VizeNativePacketFixture",
  platforms: [.macOS(.v12), .iOS(.v16)],
  products: [.library(name: "NativePacket", targets: ["NativePacket"])],
  targets: [
    .target(
      name: "NativePacket",
      path: "Sources",
      exclude: ["AppDelegate.swift", "NativeHostViewController.swift"],
      sources: ["NativeBridgePacket.swift"]
    ),
    .testTarget(name: "NativePacketTests", dependencies: ["NativePacket"], path: "PacketTests"),
  ],
  swiftLanguageVersions: [.v5]
)
