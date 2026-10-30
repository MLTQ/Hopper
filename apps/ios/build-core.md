# build-core.sh

## Purpose
Build Rust device/simulator static libraries, package an XCFramework and generate the Xcode project.

## Contracts
Run from any directory. Optional `release` argument enables release optimization. Requires Xcode,
XcodeGen, Rust targets aarch64-apple-ios and aarch64-apple-ios-sim, and fetched Cargo dependencies.
No Apple account, upload, signing, or installation is performed by this script.
The app also links Apple's Network framework, required by Iroh's iOS interface discovery.
Rust and its C dependencies use the same iOS 16 deployment target as the app.
Explicit CFLAGS also invalidate cached C objects in crates that only watch CFLAGS/CC changes.
