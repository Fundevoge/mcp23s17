# MCP23S17 driver

![Crates.io](https://img.shields.io/crates/v/mcp23s17)
![Crates.io](https://img.shields.io/crates/d/mcp23s17)
![Crates.io](https://img.shields.io/crates/l/mcp23s17)

A driver for the MCP23S17 I/O expander which is accessed over an SPI bus.

## Example usage

```rs
// spi_device implements the SpiDevice trait from embedded_hal >= 1.0.0
// The hardware address corresponds to the way the pins A0, A1, A2 are physically connected
let mut mcp23s17 = mcp23s17::Mcp23s17::new(spi_device, 0b000).ok().unwrap();

// Configure pin GPA0 as an output
mcp23s17.pin_mode(0, mcp23s17::PinMode::Output).ok().unwrap();
// Set pin GPA0 high
mcp23s17.set_high(0);

// Configure pin GPA1 as an pullup input
mcp23s17.pin_mode(1, mcp23s17::PinMode::InputPullup).ok().unwrap();
// Read GPA1's level
let is_high = mcp23s17.read(1);
```


## Example implementation of the SpiDevice trait

```rs
// Assuming the SpiBus trait is implemented for Spi

spi_device = MySpiDevice::new(spi, cs);

struct MySpiDevice {
    spi: Spi,
    cs: SpiCsPin,
}

impl MySpiDevice{
    fn new(spi: Spi, mut cs: SpiCsPin) -> MySpiDevice {
        // IMPORTANT, otherwise MCP23S17 might not work
        cs.set_high();

        MySpiDevice { spi, cs }
    }
}

impl embedded_hal::spi::SpiDevice for MySpiDevice {
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {  
        for operation in operations {
            match operation {
                embedded_hal::spi::Operation::DelayNs(_) => {}
                _ => {
                    self.cs.set_low();
                }
            }
            match operation {
                embedded_hal::spi::Operation::Read(words) => {
                    let _ = embedded_hal::spi::SpiBus::read(&mut self.spi, words);
                }
                embedded_hal::spi::Operation::Write(words) => {
                    let _ = embedded_hal::spi::SpiBus::write(&mut self.spi, words);
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    let _ = embedded_hal::spi::SpiBus::transfer(&mut self.spi, read, write);
                }
                embedded_hal::spi::Operation::TransferInPlace(words) => {
                    let _ = embedded_hal::spi::SpiBus::transfer_in_place(&mut self.spi, words);
                }
                embedded_hal::spi::Operation::DelayNs(ns) => {
                    delay_us(*ns / 1000);
                }
            }
            match operation {
                embedded_hal::spi::Operation::DelayNs(_) => {}
                _ => {
                    self.cs.set_high();
                }
            }                   
        }
        Ok(())
    }
}

impl embedded_hal::spi::ErrorType for MySpiDevice {
    type Error = Infallible;
}

impl core::fmt::Debug for MySpiDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MySpi")
    }
}
```



## Acknowledgements

Many of the documentation comments in this library are taken direct from the
[MCP23S17 datasheet](https://www.microchip.com/en-us/product/MCP23S17) and are
© 2005-2022 Microchip Technology Inc. and its subsidiaries.

Inspired by this [Arduino MCP23S17 Library](https://github.com/dreamcat4/Mcp23s17)
and the [RPPAL MCP23S17 Library](https://github.com/solimike/rppal-mcp23s17/)

