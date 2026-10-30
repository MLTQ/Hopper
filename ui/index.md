# index.html

## Purpose
Desktop library shell: navigation, collection, persistent queue, transport and modal surfaces.

## Contracts
Element IDs are consumed by app.js/player.js. Media playback uses one persistent video element
for audio and video; rerendering the collection never replaces it. No inline executable scripts
or external assets. Controls are labeled and native inputs support keyboard navigation.
