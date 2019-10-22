# impact

> A high-performance texture packer

## Installation

You can either download the latest release or build it yourself. Use `cargo build --release` to
generate an executable, or `cargo install --path .` to install it to your path.

## Usage

To view a full list of commands, run `impact --help`.

For a basic execution, run `impact --default atlas images`. This will take all files in the `images`
folder, parse them, and generate as many texture atlases as needed. The resulting atlases will be stored
as files that look like `atlas*.png` and an associated `atlas.xml` file descriptor. In addition to XML, JSON and
bincode descriptor targets are available using the `--json` and `--binary` flags, respectively.
