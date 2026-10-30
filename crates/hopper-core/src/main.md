# main.rs

## Purpose
Headless Hopper node for Linux boxes and development, with the desktop web interface included.

## Contracts
Prints one local-secret Hello JSON line; stderr explains the local URL. --import accepts repeated
local file/folder paths. --local-only disables public discovery for offline tests. CTRL-C closes
Iroh gracefully. Binding is loopback; remote devices pair over Iroh rather than exposed HTTP.
