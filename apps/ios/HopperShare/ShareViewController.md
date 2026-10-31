# ShareViewController.swift

UIKit share extension accepts one web link or OPML export through NSItemProvider. A visible Save
action writes the shared App Group inbox, then dismisses. Explicitly tells the user to open Hopper;
no unsupported background launch or network/agent execution occurs in the extension. File reads
are bounded and no arbitrary file path is forwarded to the desktop. Main app/core validate intake.
