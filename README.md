# CSS Autoprefixer

A CSS autoprefixer written in Rust. It parses CSS and automatically injects the necessary vendor prefixes (`-webkit-`, `-moz-`, etc.) so you can write standard CSS without worrying about browser compatibility.

## What it does

Write standard CSS:

```css
.card {
  transition: all 0.3s ease;
  transform: translateY(-4px);
  backdrop-filter: blur(10px);
}
```

Get cross-browser CSS:

```css
.card {
  -webkit-transition: all 0.3s ease;
  transition: all 0.3s ease;
  -webkit-transform: translateY(-4px);
  transform: translateY(-4px);
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
}
```

## How it works

The prefixer processes CSS through a three-stage pipeline:

```
CSS text  →  Lexer  →  Tokens  →  Parser  →  AST  →  Generator  →  Prefixed CSS
```

**Lexer** — Tokenizes raw CSS text into a flat stream of typed tokens (selectors, properties, values, at-rules, comments, etc.) with source position tracking.

**Parser** — Consumes the token stream and builds a hierarchical Abstract Syntax Tree (AST) that mirrors the natural nesting structure of CSS: rules contain declarations, at-rules can contain nested rules.

**Generator** — Traverses the AST and produces the final CSS string. When it encounters a declaration whose property needs vendor prefixes, it emits the prefixed variants immediately before the standard property.

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Build

```sh
cargo build
```

### Run

```sh
cargo run
```

Currently the program reads `src/input.css` and prints the prefixed output to stdout.

### Test

```sh
cargo test
```

## Project structure

```
src/
  main.rs          # Entry point — wires the pipeline together
  lexer/           # Tokenization
  parser/          # AST construction
  generator/       # CSS generation and prefix injection
  input.css        # Sample input file
```

## Roadmap

- [ ] CLI interface — accept input/output file paths as arguments
- [ ] Better CSS feature parity
- [ ] Larger test suite
