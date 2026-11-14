# podcast/timeline.rs

Pure source-to-edited timeline arithmetic. Validate the agent's source hash, measured duration,
classification/confidence and ranges before selecting cuts. Merge overlaps, complement cuts into
retained spans, and reject deletion of the entire episode. Uncertain/content segments never cut.

All timestamps use seconds. Non-overlapping fades preserve the timeline length. Chapters wholly
inside removed spans disappear; surviving boundaries shift. Chapter JSON retains artwork and
unknown fields. Transcript cues crossing cuts split into retained intervals. Soundbite durations
and saved playback positions use the same mapping. Tests cover overlap, banter, uncertainty,
chapter images, transcript splits and wrong-source rejection.

Conflicting overlapping classifications preserve protected content, uncertain ranges legacy conversation labels and retained promos. Chapter JSON entries are sorted/validated before inferred ends are mapped.

Remapped playable chapter entries carry the image URL and table-of-contents visibility from the source.

Conversation classified as legacy banter is always protected, even for older options with keep_banter set to false. Tests cover loading and serializing those preferences.
