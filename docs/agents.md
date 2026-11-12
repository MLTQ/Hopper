# Audio agents and podcast cuts

Subscribe from **Podcasts → Browse podcasts** or import an RSS link. Choose the desktop that owns the
show. On that desktop, open **Podcasts → Audio agent** to configure processing. On any paired device,
tap **Cut ads** on one episode to try the edit. This runs only that episode with its own preferences. For a backlog, open the show’s settings → **Cut ads**. New releases
are cleaned automatically only if you enable that option. The owner must remain running.

Install **FFmpeg and FFprobe** on the processing desktop's PATH. Hopper also checks standard Homebrew locations on macOS so Finder-launched apps
can find them. An installer that bundles these dependencies is follow-up packaging work.

## Built-in mode

Enter an audio-capable **Chat Completions** endpoint, its model name, and an API key if required.
The endpoint must accept `input_audio` with base64 WAV and return text; a text-only or
transcription-only endpoint is not enough. Remote endpoints require HTTPS. Local inference may
use HTTP on localhost. There is no preselected model and no model call before configuration.

Starting a cut sends the episode's audio to your chosen provider in two-minute regions with context
overlap. Provider usage charges may apply. Hopper validates the returned segment classifications,
times and confidence, checkpoints completed regions, and performs the edits locally. All agent
outputs are untrusted; uncertain segments are retained. Confidence is estimated by the model and
does not guarantee accuracy. Listen to representative edits before enabling automatic processing.

Keys are saved in a private local `agent.json`, **not an OS keychain**. They never enter peer
catalogues, request manifests, or status responses. `HOPPER_LLM_API_KEY` is an optional environment
fallback. Blank in the UI retains a saved key only when the endpoint is unchanged; the removal
checkbox clears the saved value (an environment fallback still applies).

## Hermes or a custom executable

Hermes mode accepts an executable/argv JSON array, for example `["/absolute/path/to/hermes"]`.
Hopper invokes `hermes chat --quiet -q <job instruction>` and includes the bundled hopper-cut-ads skill. Hermes needs configured model access and tools for audio listening or local timed transcription of the exact episode. A timed transcript can establish spoken sponsorships. Preserve ambiguous dialogue or music. This adapter does not install Hermes or give a
text-only model audio capabilities. A live Hermes run on an Omarchy owner desktop identified the spoken sponsor break in a short staged episode, and Hopper rendered the cut. Long episodes and more ambiguous host reads still need listening checks.

Custom mode invokes your argv followed by `--request <request.json> --output <analysis.json>`.
No shell is used. The private request includes `protocol: hopper.ad-analysis.v1`, an input audio
path, BLAKE3 source hash, measured duration, preferences and the required output location. Listen
to the actual audio, then atomically write:

```json
{
  "source_hash": "exact hash from request.json",
  "duration": 1800.0,
  "segments": [
    {"start": 120.5, "end": 150.1, "kind": "ad", "confidence": 0.97,
     "reason": "Audible sponsor introduction followed by offer and promo code"}
  ]
}
```

Times are in **original episode seconds**. Kinds are `ad`, `banter`, `promo`, `content`, `uncertain`.
An empty array is a successful analysis with no identified cuts. Failure to listen must fail the
job, never return an invented empty success. Do not modify input audio. Exit nonzero on failure;
logs remain in the job directory. Legacy `HOPPER_AD_AGENT` or `agent-command.json` argv configuration
overrides the UI until removed.

## External agents through MCP (including compatible Dot/Muse setups)

Choose **External agent · MCP inbox** in Hopper. Add the local bridge to an agent client that
supports stdio MCP, replacing the paths below with the checkout and library locations on that
desktop. The Audio agent dialog shows the jobs directory; its parent is the library directory.

```json
{
  "mcpServers": {
    "hopper": {
      "command": "python3",
      "args": ["/absolute/path/to/Hopper/scripts/hopper-agent-mcp.py",
               "--data-dir", "/absolute/path/to/Hopper-library"]
    }
  }
}
```

The bridge provides `hopper_pending_jobs`, `hopper_get_job`, `hopper_listen_clip`, `hopper_submit_analysis` and
`hopper_fail_job`. It exposes only active inbox jobs, validates source identity and ranges, and
publishes results atomically. The worker validates again before rendering. Only one configured
agent should process the inbox; there is no distributed claim/lease scheduler yet.

Point an inbox agent at `skills/hopper-cut-ads/SKILL.md`. An audio-capable client can request up to 60 seconds of actual WAV audio per call, including when the bridge runs on the owner over SSH. Ask the agent to process the chosen episode and submit its analysis. To work
unattended, configure a polling/watch task in that agent app. **MCP provides the tools, not an
automatic wakeup.** Dot and Muse have no verified app-specific launcher in this implementation;
compatibility depends on their actual MCP and audio-tool support. No settings in those apps are
changed by Hopper. The bridge negotiates MCP through version 2025-11-25.

## Edits, recovery and limits

Jobs persist across restart. Cancellation stops analysis and prevents publishing an in-progress
render. An external analyzer has a four-hour limit; FFmpeg rendering has a separate limit.
Original downloads are immutable. Cleaned variants use AAC/M4A, retain separate IDs, and become
playable only after validation. A failed recut retains the previously ready variant. New play/queue
actions prefer the ready cleaned variant; an already-playing/queued original is not switched mid-play.

Chapter timing, chapter JSON image/link fields, WebVTT/SRT cues, soundbites and saved position are
mapped through the same edit timeline. Chapter JSON/transcript results and the edit timeline remain
in the private job manifest. The app uses remapped chapters; source container chapters are stripped
from the M4A rather than leaving stale times. Exporting embedded chapters and a cleaned RSS feed,
and full Podcasting 2.0 coverage are still follow-up work. The player uses available chapter artwork.

Apple Podcasts and Podcast Index discovery, direct RSS, Apple show links, and OPML imports use the same desktop subscription flow. The first
play downloads an episode to its owning desktop before playback. Live streams are not edited.
Cut ads currently handles audio episodes. UI and integration tests use synthetic fixtures; real
model accuracy, long-episode costs, and third-party agent compatibility need separate evaluation.
