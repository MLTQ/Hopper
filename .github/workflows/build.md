# build.yml

Build/test matrix for macOS, Ubuntu and Windows desktops plus an unsigned iOS simulator build.
Install FFmpeg and Python before core tests: podcast integration requires actual rendering and a
synthetic external agent/MCP bridge. Linux also installs Tauri webview dependencies. Artifacts are
development executables, not signed installers. This workflow is checked in but has not run on a
remote CI service; local platform checks remain the only claimed results.
