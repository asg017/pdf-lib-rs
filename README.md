# `pdf-lib-rs`

A vibe-coded Rust port of [`pdf-lib`](https://github.com/Hopding/pdf-lib), for a few reasons:

- I wanted a low/no dependency PDF library for a few different projects
- Wanted something that works in single-binary CLIs, WASM builds,

The code hasn't been audited or looked at closely. I would say that it does work for PDFs I care about, but other PDFs may not work as well.

I don't plan on making this a 100% compatible library, or work for 100% of the PDFs that work with the original `pdf-lib`. But it's "good enough" for my non critical needs.

Use at your own risk!