# PlaybackDownloads.swift

Start a verified download for each remote item when it becomes current. Reuse its download task
if AVFoundation reports a streaming failure. Recovery opens the verified local file at the
current position and checks the active item before replacing it. Saving status is visible in
the player. A download failure keeps a healthy stream playing.
