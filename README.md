# jimg

## Operations

Rhai scripts receive `source` (the input image) and `t` (normalized animation
progress from `0.0` to `1.0`) and must return an `Image` as their final value.
Image operations return a new image, so variables can hold and reuse
intermediate results.

### Filters

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
| `img + img` | Two images to sum | Sums each pixel component of each image |
| `img - img` | Two images to sub | Subtracts each pixel component of the right from the left |

### Useful functions
| Function | Arguments | Returns |
| `smoothstep(start, end, t)` | Smooths t between start and end | start if t<=start, end if t>=end, otherwise a value between them |

## Build

```sh
cargo build --release
```

## Usage

```sh
jimg --input <INPUT> [--file <SCRIPT>] [--time <TIME>] [--output <OUTPUT>]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `-i, --input <INPUT>` | Path to the source image |
| `-o, --output <OUTPUT>` | Path to write the result (default: `output.gif`) |
| `-t, --time <TIME>` | Amount of milliseconds the gif should run for (default: `1000`) |
| `-f, --file` | Path to a Rhai script (default: `filter.rhai`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

### Examples

Use a Rhai script to animate a spin

```sh
jimg -i photo.png -t 1000 -f timelines/spin.rhai
```

`spin.rhai`:
```rhai
source.rotate(360.0 * t)
```

Rhai scripts can use the image passed by `--input`, save intermediate images,
and return their finished image:

```rhai
let green_only = source.multiply_all(0.0, 1.0, 0.0);
let smaller = source.scale(1.0 - 0.5 * t, 1.0);
smaller.rotate(360.0 * t) - green_only
```

Basic GIF creation still supports inline operations:

## Notes

- Rhai scripts are evaluated for each animation frame with the same source image and a changing `t`.
- Applying filters in Rhai scripts is lazy. If an image is not used in the final output, filters applied to it will not be processed.
- `--time` sets the GIF duration in milliseconds.
- Example scripts can be found in `timelines/`.
- Gifs always run at 15ms. I'll change this if it ever becomes a problem.
