# MCP23S17 driver

![Crates.io](https://img.shields.io/crates/v/mcp23s17)
![Crates.io](https://img.shields.io/crates/d/mcp23s17)
![Crates.io](https://img.shields.io/crates/l/mcp23s17)

A driver for the MCP23S17 I/O expander which is accessed over an SPI bus.

## Example usage

```rs
// spi implements the SPI traits from embedded_hal 
// cs_pin implements the OutputPin embedded_hal
// The hardware address corresponds to the way the pins A0, A1, A2 are physically connected
let mut mcp23s17 = mcp23s17::Mcp23s17::new(spi, cs_pin, 0b000).ok().unwrap();

// Configure pin GPA0 as an output
mcp23s17.pin_mode(0, mcp23s17::PinMode::Output).ok().unwrap();
// Set pin GPA0 high
mcp23s17.set_high(0);

// Configure pin GPA1 as an pullup input
mcp23s17.pin_mode(1, mcp23s17::PinMode::InputPullup).ok().unwrap();
// Read GPA1's level
let is_high = mcp23s17.read(1);
```

## Acknowledgements

Many of the documentation comments in this library are taken direct from the
[MCP23S17 datasheet](https://www.microchip.com/en-us/product/MCP23S17) and are
© 2005-2022 Microchip Technology Inc. and its subsidiaries.

Inspired by this [Arduino MCP23S17 Library](https://github.com/dreamcat4/Mcp23s17)
and the [RPPAL MCP23S17 Library](https://github.com/solimike/rppal-mcp23s17/)

