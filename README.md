# Moth

**Moth** is a fast CLI tool for converting `.moth` files — a markdown-inspired format — into HTML. It supports inline math, embedded images, raw HTML blocks, and live reload for rapid editing.


## Usage

### Transpile a `.moth` file into HTML

```sh
moth transpile myfile.moth
```

Options:

- `--no-base-64` — disables image base64 embedding (uses file references instead)
- `--out-file <file>` — sets the output HTML file name (default: `output.html`)

### Live-reload server

```sh
moth serve myfile.moth
```

Options:

- `--ws-port <port>` — websocket port (default: 8000)
- `--http-port <port>` — HTTP port (default: 8001)

## Syntax

The Moth syntax is close to Markdown. Documentation for it will be added here soon.

## Installation

Installation instructions are in the making.

## Contributing

If you'd like to contribute to Moth, feel free to check out the repository. However, please note that feature requests are not being accepted at this time. If you're interested in helping with improvements or fixes, your contributions are welcome!
