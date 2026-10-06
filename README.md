# jimg

## Operations

| Operation | Operands | Effect |
|-----------|----------|--------|
| `scale`  | x_factor: scales on width, y_factor: scales on height | Scales the image |
| `slant` | angle: angle to perform the slant at | Shears the image |
| `flip-horizontal` | | Flips the image horizontally |
| `flip-vertical` | | Flips the image vertically |
| `rotate`   | angle: angle to rotate image to in degrees, clockwise | Rotates the image |
| `raw-resize` | x: new width, y: new height | Changes the size of an image but not the positions of any pixels |
| `resize` | x: new width, y: new height | Scales an image to a new size, stretching as neccessary |

## Build

```sh
cargo build --release
```

## Usage

```sh
jimg --input <INPUT> --time <TIME> [--output <OUTPUT>] (file <TIMELINE>|<OPERS>)
```

### Arguments

| Argument | Description |
|----------|-------------|
| `-i, --input <INPUT>` | Path to the source image |
| `-o, --output <OUTPUT>` | Path to write the result (default: `output.gif`) |
| `-t, --time <TIME>` | Amount of milliseconds the gif should run for |
| `-h, --help` | Print help |
| `-V, --version` | Print version |
| `<TIMELINE>` | Path to the timeline TOML file |
| `<OPERS>` | List of operations seperated by the + character |

### Examples

Halve an image's width, then spin it in a circle

```sh
jimg -i photo.png -t 1000 -o squished.gif scale --x_factor 1..0.5 --y_factor 1 --end 0.5 --ease cubic + rotate --angle 0..360 --start 0.5
```

Use the modifications specified in timeline.toml

```sh
jimg -i photo.png -t 1000 file timeline.toml
```

Timeline file that rotates an image CCW and then back

```toml
length = 1000
input = "photo.png"

[[effects]]
op = "rotate"
angle = "0..360"
end = 0.5
ease = "quad"

[[effects]]
op = "rotate"
angle = "0..-360"
start = 0.5
```

## Supported formats

## Notes

- When passing a timeline file, input and output files from the command line are prioritized
- In fact, input and output are not neccessary in the timeline file, opting to be always specified in the command
- Example timelines can be found in timelines/
