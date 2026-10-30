# api.js

## Purpose
Authenticated local API client plus pure display formatting/escaping helpers.

## Contracts
Bearer token arrives via a shell-provided URL fragment, is scoped to this origin's session storage,
and is removed from the visible URL. Secrets never enter metadata or peer requests. All imported
titles/filenames must be escaped before insertion into HTML. Requests surface JSON errors.
