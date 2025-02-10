# micro:bit Button Press Indicator

On the [micro:bit hardware details page](https://tech.microbit.org/hardware/), I saw

> Front buttons A and B ... are debounced by software, which also includes short press, long press, and ‘both A+B’ press detection.

So, why not write an async button press detection program as a practice?

## Requirement

This project uses ***micro:bit v2***, not ***v1***.

When building, make sure `probe-rs` and target `thumbv7em-none-eabihf` are installed.

Install them with

```zsh
cargo install probe-rs-tools
cargo target thumbv7em-none-eabihf
```

## Building and Running

1. Plug the board to your computer

2. Run the command

```zsh
cargo embed --release
```
