# blog.rs

![CI](https://img.shields.io/github/actions/workflow/status/ae2rs/blog.rs/ci.yml?branch=main)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/rust-2024%20edition-D16A2A.svg)

Live: [decastro.dev](https://decastro.dev) · [ae2.rs](https://ae2.rs)

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
- `styles/index.css`: Tailwind entrypoint (compiled to `build/style/index.css`)
- `build/`: static assets served at runtime (e.g., `build/img` and `build/style`)
- `content/`: blog content source files

## Quickstart

```sh
make run
```

Then open `http://0.0.0.0:3000`.

## Local development

`make` lists every target. The one you want day to day is:

```sh
make dev
```

It rebuilds and restarts the server whenever Rust, content, styles, or assets change. Posts are
baked in at compile time by the `Post` derive macro, so a content edit means a full rebuild — there
is no in-process reload. Requires `cargo-watch` (`cargo install cargo-watch`); the browser still
needs a manual refresh.

## Build

```sh
make build      # debug
make release    # optimized
```

Both run `build.rs`, which generates the minified Tailwind CSS using the binary in `vendor/tailwind/`.

## Test

```sh
make test
```

## Lint

```sh
make fmt        # format in place
make lint       # clippy, warnings denied
make ci         # fmt-check + check + lint + test, same as the CI workflow
```

## Docker

```sh
make up         # docker compose up --build
make down       # docker compose down
```

## License

MIT. See `LICENSE`.

## TODO

- View transitions support for the navbar
- Caching headers
- About section for the blog itself
- Notes (tiny posts) support
- Analytics
