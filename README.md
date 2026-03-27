# mdlite

`mdlite` is a super-lightweight terminal Markdown reader written in Rust.

It focuses on the basic features of GitHub Flavored Markdown and keeps the implementation dependency-free.

## Usage

```bash
mdlite README.md
cat README.md | mdlite
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
