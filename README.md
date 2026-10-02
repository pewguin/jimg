# jimg

## Operations

| Operation | Modifier | Effect |
|-----------|----------|--------|
| `squish`  | Squish factor (e.g. `2`) | Narrows the image horizontally. |
| `slant`   | Angle in degrees (e.g. `30`) | Shears the image horizontally. |

## Build

```sh
cargo build --release
```

## Usage

```sh
jimg <OPER> <MODIFIER> --input <INPUT> [--output <OUTPUT>]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `OPER` | Operation to apply: `squish` or `slant` |
| `MODIFIER` | Float value used by the operation (squish factor or slant angle in degrees) |
| `-i, --input <INPUT>` | Path to the source image |
| `-o, --output <OUTPUT>` | Path to write the result (default: `output.png`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

### Examples

Squish an image to half its width:

```sh
jimg squish 2 -i photo.png -o squished.png
```

Slant an image by 30 degrees:

```sh
jimg slant 30 -i photo.png -o slanted.png
```

## Supported formats

Input and output formats are handled by the [`image`](https://crates.io/crates/image) crate. The output format is chosen from the file extension of `--output` (for example `.png`, `.jpg`, `.bmp`).

## Notes

- Giving `slant` angle values outside of [0, 90) will cause issues. As the input approaches 90, the image will get bigger (see f(x)=tan(x))
- `squish` has no interpolation
