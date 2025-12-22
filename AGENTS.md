# Repository Guidelines

## Project Structure & Module Organization
- `bin/main.rs` starts the Axum server and wires routes and static assets.
- `src/lib.rs` exposes the library crate; `src/pages/` holds page handlers and Maud templates.
- `styles/index.css` is the Tailwind entry file; compiled output is written to `public/style/index.css`.
- `public/` contains static assets served at runtime (e.g., `public/img/` and `public/style/`).
- `vendor/tailwind/` stores prebuilt Tailwind binaries used by `build.rs`.

## Build, Test, and Development Commands
- `cargo build` compiles the server and runs `build.rs`, which generates the minified Tailwind CSS.
- `cargo run` builds and launches the server at `http://0.0.0.0:3000`.
- `cargo test` runs Rust tests (none are present yet; add tests as the project grows).

## Coding Style & Naming Conventions
- Use standard Rust formatting (`cargo fmt`), 4-space indentation, and `snake_case` for modules/functions.
- Prefer `UpperCamelCase` for types and `SCREAMING_SNAKE_CASE` for constants.
- Keep templates in `src/pages/` and match handler names to routes (e.g., `index` for `/`).
- CSS changes should go in `styles/index.css`; do not edit `public/style/index.css` directly.

## Testing Guidelines
- Add unit tests in-module with `#[cfg(test)]` and `mod tests { ... }`.
- Name test functions with clear behavior (e.g., `renders_index_page`).
- Run `cargo test` before submitting changes.

## Commit & Pull Request Guidelines
- Recent commits use short, lowercase, past-tense sentences (e.g., "added support for tailwind"); follow that style.
- PRs should include a concise description, test notes (`cargo test`, `cargo run`), and screenshots for UI changes.

## Configuration Notes
- Tailwind is built via `build.rs` using the binary in `vendor/tailwind/`; ensure the correct platform binary exists before building.
