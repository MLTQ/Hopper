# index.html

## Purpose
Desktop library shell: navigation, collection, persistent queue, transport and modal surfaces.

## Contracts
Element IDs are consumed by app.js/player.js. Media playback uses one persistent video element
for audio and video; rerendering the collection never replaces it. No inline executable scripts
or external assets. Controls are labeled and native inputs support keyboard navigation.
Podcasts is a first-class navigation destination; its show/settings surfaces reuse the shared dialog.

The existing avatar is an accessible Appearance button. material.css layers material tokens over
styles.css without changing geometry. The system light/dark color scheme is advertised.

The Folders view has a labeled Added folders region above the collection filters.
