# NUCLEO-L452RE-P Embassy Examples

This repository contains code examples using Embassy for the NUCLEO-L452RE-P development board.

# Requirements

- Install [rustup](https://rustup.rs/)

- Install the cross compiler target

```bash
rustup target add thumbv7em-none-eabi
```

- Install `probe-rs` with defmt support.

```bash
cargo install probe-rs --features=cli
```

## Running test example code

For example:

```bash
cargo run --bin nuc-test-blinky
```

## Flashing test example code

```bash
cargo embed --bin nuc-test-blinky
```

## Examples confirmed to be working on the nucleo dev board

- [x] adc
- [x] blinky
- [x] boot (board independent)
- [x] button_exti
- [x] button
- [x] dac_dma
- [x] dac
- [ ] flash_async (board independent)
- [x] flash (board independent)
- [x] i2c_blocking_async
- [x] i2c_dma
- [x] i2c
- [x] mco
- [x] rng (board independent)
- [x] rtc (board independent)
- [x] spi_blocking_async
- [x] spi_dma
- [x] spi
- [x] usart_dma
- [x] usart
- [ ] usb_serial (does not enumerate)

