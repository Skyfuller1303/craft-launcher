# English UI evidence for draft PR #141

The six app screenshots were rendered from c4989a87859c061fd5f9dd0058e2fe078d72be76.
The four launcher settings screenshots were rendered from 88ed3bd1352c88e7f42022c4441c89ec14c20823.

All images exercise the real egui UI with English explicitly selected and synthetic
release and installation data. No installed applications were changed for screenshots.
Generated PNGs live on this evidence branch to keep them out of the production diff.

Local validation: cargo xtask ci passed fmt, Clippy, tests (257 passed, 2 existing ignored)
and Wasm checks. Native installer end-to-end validation remains pending as documented in PR.