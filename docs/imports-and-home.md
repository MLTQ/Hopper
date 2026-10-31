# Bring your podcasts home

## Choose your Home desktop

1. Run Hopper on the desktop that should store media and run the audio agent.
2. Pair your laptop and iPhone directly with that desktop using Devices → Create invitation.
3. On the laptop and iPhone, choose that desktop under Devices → Home desktop.
4. Configure Podcasts → Audio agent on Home. Per-show Cut ads settings and jobs run on the
   desktop that owns the show. Selecting Home does not move existing subscriptions or configure
   an agent automatically.

Each desktop can still play media. New subscriptions and file imports go to the selected Home;
there is no need to pair the phone with the laptop just to play files delivered to Home.
Home must keep Hopper running (or run hopper-node). Automatic startup/service installation is
not included. Downloads kept on the phone play while Home is unavailable.

## Apple Podcasts

Share a show or episode link into Hopper, or copy the link and paste it in Podcasts → ＋.
Hopper uses Apple's public lookup service to find the RSS feed. A show link subscribes on Home;
an episode link also saves that episode if its GUID or enclosure matches the publisher's feed.
Apple’s episode listing is bounded to 200 items. If an older episode is absent or cannot be matched, the delivery message says so and you can choose from the show's episodes.
Paid/private Apple-only content without a public feed cannot be imported by this route.
There is no automatic reading of another app's private library, downloads or listening history.

## Castamatic

Castamatic supports OPML subscription export. Export your list, save the file to Files, then use
Hopper's Podcasts → ＋ → Import subscription list. Desktop Hopper accepts the same OPML file.
Nested folders are traversed; repeated feed URLs in a file are deduplicated. The first version
imports the subscriptions, not Castamatic folder organization, history, queue or downloaded audio.
Lists are limited to 2 MB and 500 feeds. Invalid feed addresses are skipped with a visible warning.

## Sharing on iPhone

The share extension accepts a web link or OPML export and saves it to Hopper's shared inbox.
**Open Hopper afterward** to send it to Home. A desktop/Home must be selected first. The main app
moves each share into its persistent delivery queue; receipts prevent duplicate processing on
replay. Invalid shared inputs remain visible in Devices → Shared imports waiting for retry/discard.

Physical-device builds need the App Group `group.app.hopper.player` enabled in both the Hopper
app and HopperShare extension provisioning profiles. Configure your own development team in
`apps/ios/project.yml`, then regenerate. Manual link paste and file import work without the
extension. No background launch, transfer while terminated, or automatic app opening is promised.

## While devices are offline

Devices and Podcasts show Sending to Home. Pending additions remain on the sending device and
retry while Hopper runs. Interrupted sends recover after restart; Home acknowledges only after
saving the subscription or verifying and retaining a media copy. Explicit feed/permission errors
remain visible with Retry. A saved receipt makes re-sending an accepted job harmless.

For files, the sender first makes a managed copy without modifying the source. Home receives only
an item ID published in the paired sender's library, never a filesystem path. After confirmation,
the laptop can go offline and Home can serve the file. Copies on the sender are retained; no
automatic cleanup, folder watching, full replication, or device-wide queue synchronization occurs.
A transfer needs the source app running until Home has the bytes. Changing Home affects future
additions and does not redirect pending jobs or migrate existing shows.

Spotify integration is deferred. Use the publisher's RSS feed when available.

## References

- [Castamatic feature list](https://castamatic.com/) — OPML import/export.
- [Apple lookup API](https://developer.apple.com/library/archive/documentation/AudioVideo/Conceptual/iTuneSearchAPI/LookupExamples.html).
- [Apple share extensions](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/Share.html).
