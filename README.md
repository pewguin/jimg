# jimg

## Operations

Rhai scripts receive `source` (the input image) and `t` (normalized animation
progress from `0.0` to `1.0`) and must return an `Image` as their final value.
Image operations return a new image, so variables can hold and reuse
intermediate results:

| Operation | Arguments | Effect |
|-----------|-----------|--------|
| `scale(x, y)` | width and height factors | Scales the image |
| `shear(horizontal, vertical)` | horizontal and vertical shear | Shears the image |
| `flip_horizontal()` | | Flips the image horizontally |
| `flip_vertical()` | | Flips the image vertically |
| `rotate(degrees)` | angle in degrees, clockwise | Rotates the image |
| `raw_resize(width, height)` | output dimensions | Changes the canvas dimensions without scaling pixels |
| `resize(width, height)` | output dimensions | Scales the image to the new dimensions |
| `multiply(factor)` | color multiplier | Multiplies image color values |
| `multiply_all(r, g, b)` | per-channel multipliers | Multiplies RGB channels |

## Build

```sh
cargo build --release
```

## Usage

```sh
jimg --input <INPUT> --time <TIME> [--output <OUTPUT>] (file <SCRIPT>|<OPERS>)
```

### Arguments

| Argument | Description |
|----------|-------------|
| `-i, --input <INPUT>` | Path to the source image |
| `-o, --output <OUTPUT>` | Path to write the result (default: `output.gif`) |
| `-t, --time <TIME>` | Amount of milliseconds the gif should run for |
| `-h, --help` | Print help |
| `-V, --version` | Print version |
| `<SCRIPT>` | Path to a Rhai script |
| `<OPERS>` | List of operations seperated by the + character |

### Examples

Halve an image's width, then spin it in a circle

```sh
jimg -i photo.png -t 1000 -o squished.gif scale --x_factor 1..0.5 --y_factor 1 --end 0.5 --ease cubic + rotate --angle 0..360 --start 0.5
```

Use a Rhai script to animate a spin

```sh
jimg -i photo.png -t 1000 file timelines/spin.rhai
```

Rhai scripts can use the image passed by `--input`, save intermediate images,
and return their finished image:

```rhai
let original = source.clone();
let smaller = original.scale(1.0 - 0.5 * t, 1.0);
smaller.rotate(360.0 * t)
```

Basic GIF creation still supports inline operations:

```sh
jimg -i photo.png -t 1000 scale --x-factor 1..0.5 --y-factor 1 + rotate --angle 0..360
```

## Supported formats

## Notes

- Rhai scripts are evaluated for each animation frame with the same source image and a changing `t`.
- `--time` sets the GIF duration in milliseconds.
- Example scripts can be found in `timelines/`.
