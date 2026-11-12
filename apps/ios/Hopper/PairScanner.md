# PairScanner.swift

AVFoundation reads QR metadata on-device after camera permission. The camera session starts and
stops on one serial queue. Only one bounded Hopper invitation is handed to the existing pairing
API per sheet, and unrelated QR codes leave scanning active. Denial or unavailable hardware
provides a route back to manual paste. Camera frames and invitation contents are never logged.
