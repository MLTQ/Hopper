# hopper_core.h

## Purpose
C ABI shared by Swift and the Rust static library.

## Contracts
Arguments are NUL-terminated UTF-8, return is owned JSON. Free once with hopper_free.
Start off the main thread. A second call returns the existing instance's Hello.
