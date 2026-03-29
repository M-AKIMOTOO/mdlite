# mdlite

`mdlite` is a super-lightweight terminal Markdown reader written in Rust.

It focuses on the basic features of GitHub Flavored Markdown and keeps the implementation dependency-free.

## Installation

```bash
cargo install mdlite
```

## Usage

```bash
mdlite README.md
cat README.md | mdlite
```

## Library usage

```rust
let rendered = mdlite::render_markdown("# Hello\n\n- one\n- two\n", false);
print!("{rendered}");
```

ANSI styling can be enabled by passing `true` as the second argument.

```rust
let rendered = mdlite::render_markdown("`code` and **bold**\n", true);
```

To page the rendered output inside a terminal:

```rust
let rendered = mdlite::render_markdown("# Document\n", true);
mdlite::run_pager(&rendered)?;
# Ok::<(), String>(())
```

## Supported features

- ATX headings (`#`, `##`, ...)
- Setext headings (`===`, `---`)
- paragraphs
- fenced code blocks
- block quotes
- unordered and ordered lists
- task list items
- thematic breaks
- basic GFM-style tables
- inline code
- emphasis and strong emphasis
- strikethrough
- links rendered as `label <url>`

## Notes

- This is intentionally not a full CommonMark/GFM implementation.
- The feature set is based on the block and inline categories listed in the GFM spec:
  https://github.github.com/gfm/
