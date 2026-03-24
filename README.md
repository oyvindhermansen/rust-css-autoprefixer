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

## How It Works

The autoprefixer runs CSS through three stages: a **lexer**, **parser**, and **generator**.

### Lexer

The lexer tokenizes raw CSS into a flat list of typed tokens:

**Input**
```css
.card {
  transition: all 0.3s ease;
}
```

**Output**
```
ClassSelector  ".card"
Whitespace     " "
CurlyOpen      "{"
Whitespace     "\n  "
Identifier     "transition"
Colon          ":"
Whitespace     " "
Identifier     "all"
Whitespace     " "
Dimension      "0.3s"
Whitespace     " "
Identifier     "ease"
Semicolon      ";"
Whitespace     "\n"
CurlyClose     "}"
```

### Parser

The parser consumes tokens and builds a typed AST:

**Output**
```
Rule { selector: ".card" }
└── Declaration { property: "transition", value: "all 0.3s ease" }
```

### Generator

The generator walks the AST and outputs prefixed CSS:

**Output**
```css
.card {
  -webkit-transition: all 0.3s ease;
  transition: all 0.3s ease;
}
```

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
