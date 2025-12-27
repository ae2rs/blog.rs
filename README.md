# blog.rs

![CI](https://img.shields.io/github/actions/workflow/status/ae2rs/blog.rs/ci.yml?branch=main)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/rust-2024%20edition-000000.svg)

Public repository for my blog, fully written in Rust. The server is built with Axum, templates are rendered with Maud, and styles are generated with Tailwind during `build.rs`.

## Features

- Axum server with Maud templates
- Markdown content rendering
- Syntax highlighting via syntect
- Embedded static assets
- Tailwind CSS build pipeline

## Project layout

- `main.rs`: server entry point and routing
- `src/`: library crate with page handlers and templates
- `styles/index.css`: Tailwind entrypoint (compiled to `public/style/index.css`)
- `public/`: static assets served at runtime
- `content/`: blog content source files

## Quickstart

```sh
cargo run
```

Then open `http://0.0.0.0:3000`.

## Build

```sh
cargo build
```

This runs `build.rs`, which generates the minified Tailwind CSS using the binary in `vendor/tailwind/`.

## Test

```sh
cargo test
```

## Lint

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Docker

```sh
docker compose up --build
```

## License

MIT. See `LICENSE`.

## TODO

- View transitions support for the navbar
- Caching headers
- About section for the blog itself
- Notes (tiny posts) support
- Analytics
