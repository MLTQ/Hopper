#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
root="$(cd ../.. && pwd)"
configuration="${1:-debug}"
export IPHONEOS_DEPLOYMENT_TARGET=16.0
hopper_cflags="${CFLAGS:-}"
cargo_flags=()
if [[ "$configuration" == release ]]; then cargo_flags+=(--release); fi
mkdir -p build
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
  if [[ "$target" == *-sim ]]; then
    export CFLAGS="$hopper_cflags -mios-simulator-version-min=16.0"
  else
    export CFLAGS="$hopper_cflags -miphoneos-version-min=16.0"
  fi
  cargo build --manifest-path "$root/Cargo.toml" -p hopper-core --lib --target "$target" "${cargo_flags[@]}" --locked --offline
done
if [[ -d build/HopperCore.xcframework ]]; then mv build/HopperCore.xcframework "build/HopperCore.previous.$(date +%s).xcframework"; fi
xcodebuild -create-xcframework \
  -library "$root/target/aarch64-apple-ios/$configuration/libhopper_core.a" -headers include \
  -library "$root/target/aarch64-apple-ios-sim/$configuration/libhopper_core.a" -headers include \
  -output build/HopperCore.xcframework
xcodegen generate
