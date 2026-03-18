# picascoo

Convert your images and videos to ASCII art right in your terminal.

## Prereqs

You'll need `ffmpeg` in your `$PATH` (and Rust toolchain of course)

## Quickstart

```console
cargo run -- ./examples/kitty.mp4 1000
```
where the 2nd arg is output width in character

## On moving ffmpeg from compile- to run-time dependency

`picascoo` originally use the `ffmpeg-next` crate to link `ffmpeg` as a library at build time. This has been working fine for me on Linux (Void Linux specifically) but turned into a mess on macOS -- `bindgen` uses clang internally to parse FFmpeg's C headers, and clang kept looking for them in `/usr/include` without success. Homebrew puts them elsewhere and none of the env vars I know could override the hardcoded path in `build.rs`

I've put up with it in several trials, and now I just pipe raw frames from the `ffmpeg` cli at runtime. This way `cargo`needs zero C dependencies and works everywhere (unconfirmed for Windows yet)
