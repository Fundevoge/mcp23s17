//! # MCP23S17 driver
//!
//! ![Crates.io](https://img.shields.io/crates/v/mcp23s17)
//! ![Crates.io](https://img.shields.io/crates/d/mcp23s17)
//! ![Crates.io](https://img.shields.io/crates/l/mcp23s17)
//!
//! A driver for the MCP23S17 I/O expander which is accessed over an SPI bus.
//!
//! ## Example usage
//!
//! ```
//! // spi implements the SPI traits from embedded_hal
//! // cs_pin implements the OutputPin embedded_hal
//! // The hardware address corresponds to the way the pins
//! // A0, A1, A2 are physically connected
//! let mut mcp23s17 = mcp23s17::Mcp23s17::new(spi, cs_pin, 0b000).ok().unwrap();
//!
//! // Configure pin GPA0 as an output
//! mcp23s17.pin_mode(0, mcp23s17::PinMode::Output).ok().unwrap();
//! // Set pin GPA0 high
//! mcp23s17.set_high(0);
//!
//! // Configure pin GPA1 as an pullup input
//! mcp23s17.pin_mode(1, mcp23s17::PinMode::InputPullup).ok().unwrap();
//! // Read GPA1's level
//! let is_high = mcp23s17.read(1);
//! ```
//!
//! ## Acknowledgements
//!
//! Many of the documentation comments in this library are taken direct from the
//! [MCP23S17 datasheet](https://www.microchip.com/en-us/product/MCP23S17) and are
//! © 2005-2022 Microchip Technology Inc. and its subsidiaries.
//!
//! Inspired by this [Arduino MCP23S17 Library](https://github.com/dreamcat4/Mcp23s17)
//! and the [RPPAL MCP23S17 Library](https://github.com/solimike/rppal-mcp23s17/)

#![no_std]
#![deny(missing_docs)]

use bitflags::bitflags;

use core::result;

type Result<T> = result::Result<T, Mcp23s17SpiError>;

/// The register addresses within the device.
///
/// Note that this follows the "interleaved" format for the register addresses so that
/// the [`IOCON::BANK`] bit of [`IOCON`][`RegisterAddress::IOCON`] register must be set
/// to 0 ([`IOCON::BANK_OFF`]).
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RegisterAddress {
    /// I/O direction A
    IODIRA = 0x0,
    /// I/O direction B
    IODIRB = 0x1,
    /// I/O polarity A
    IPOLA = 0x2,
    /// I/O polarity B
    IPOLB = 0x3,
    /// interrupt enable A
    GPINTENA = 0x4,
    /// interrupt enable B
    GPINTENB = 0x5,
    /// register default value A (interrupts)
    DEFVALA = 0x6,
    /// register default value B (interrupts)
    DEFVALB = 0x7,
    /// interrupt control A
    INTCONA = 0x8,
    /// interrupt control B
    INTCONB = 0x9,
    /// I/O config (also at 0xB)
    IOCON = 0xA,
    /// I/O config (duplicate)
    IOCON2 = 0xB,
    /// port A pull-ups
    GPPUA = 0xC,
    /// port B pull-ups
    GPPUB = 0xD,
    /// interrupt flag A (where the interrupt came from)
    INTFA = 0xE,
    /// interrupt flag B
    INTFB = 0xF,
    /// interrupt capture A (value at interrupt is saved here)
    INTCAPA = 0x10,
    /// interrupt capture B
    INTCAPB = 0x11,
    /// port A
    GPIOA = 0x12,
    /// port B
    GPIOB = 0x13,
    /// output latch A
    OLATA = 0x14,
    /// output latch B
    OLATB = 0x15,
}

bitflags! {
    /// I/O Expander Configuration Register (`IOCON`) bit definitions.
    pub struct IOCON: u8 {
        /// Controls how the registers are addressed:
        ///
        ///   1 = The registers associated with each port are separated into different
        ///       banks. (*Not currently supported in this library.*)
        ///
        ///   0 = The registers are in the same bank (addresses are sequential).
        const BANK = 0b1000_0000;

        /// `INT` Pins Mirror bit:
        ///
        ///   1 = The `INT` pins are internally connected.
        ///
        ///   0 = The `INT` pins are not connected. `INTA` is associated with `PORTA`
        ///       and `INTB` is associated with `PORTB`.
        const MIRROR = 0b0100_0000;

        /// Sequential Operation mode bit:
        ///
        ///   1 = Sequential operation disabled, address pointer does not increment.
        ///
        ///   0 = Sequential operation enabled, address pointer increments.
        const SEQOP = 0b0010_0000;

        /// Slew Rate control bit for SDA output:
        ///
        ///   1 = Slew rate control disabled.
        ///
        ///   0 = Slew rate control enabled.
        const DISSLW = 0b0001_0000;

        /// Hardware Address Enable bit:
        ///
        ///   1 = Enables the MCP23S17 address pins.
        ///
        ///   0 = Disables the MCP23S17 address pins.
        const HAEN = 0b0000_1000;

        /// Configures the `INT` pin as an open-drain output:
        ///
        ///   1 = Open-drain output (overrides the `INTPOL` bit.)
        ///
        ///   0 = Active driver output (`INTPOL` bit sets the polarity.)
        const ODR = 0b0000_0100;

        /// Sets the polarity of the `INT` output pin:
        ///
        ///   1 = Active-high.
        ///
        ///   0 = Active-low.
        const INTPOL = 0b0000_0010;

        /// Unimplemented: Read as 0.
        const _NA = 0b0000_0001;
    }
}

impl IOCON {
    /// The registers associated with each port are separated into different
    /// banks. (*Not currently supported in this library.*)
    pub const BANK_ON: IOCON = IOCON::BANK;
    /// The registers are in the same bank (addresses are interleaved sequentially).
    pub const BANK_OFF: IOCON = IOCON::empty();
    /// The `INT` pins are internally connected.
    pub const MIRROR_ON: IOCON = IOCON::MIRROR;
    /// The `INT` pins are not connected. `INTA` is associated with `PORTA` and `INTB`
    /// is associated with `PORTB`.
    pub const MIRROR_OFF: IOCON = IOCON::empty();
    /// Sequential operation enabled, address pointer increments.
    pub const SEQOP_ON: IOCON = IOCON::empty();
    /// Sequential operation disabled, address pointer does not increment.
    pub const SEQOP_OFF: IOCON = IOCON::SEQOP;
    /// Slew rate control enabled.
    pub const DISSLW_SLEW_RATE_CONTROLLED: IOCON = IOCON::empty();
    /// Slew rate control disabled.
    pub const DISSLW_SLEW_RATE_MAX: IOCON = IOCON::DISSLW;
    /// Enables the MCP23S17 address pins.
    pub const HAEN_ON: IOCON = IOCON::HAEN;
    /// Disables the MCP23S17 address pins.
    pub const HAEN_OFF: IOCON = IOCON::empty();
    /// Open-drain output (overrides the `INTPOL` bit.)
    pub const ODR_ON: IOCON = IOCON::ODR;
    /// Active driver output (`INTPOL` bit sets the polarity.)
    pub const ODR_OFF: IOCON = IOCON::empty();
    /// Active-high.
    pub const INTPOL_HIGH: IOCON = IOCON::INTPOL;
    /// Active-low.
    pub const INTPOL_LOW: IOCON = IOCON::empty();
}

/// The only error that operation of the MCP23S17 can raise, because of an internal `SPI` error.
#[derive(core::fmt::Debug)]
pub struct Mcp23s17SpiError {}

/// The possible configurations of a pin. [PinMode::InputPullup]
/// uses a 100k internal pullup resistor.
#[repr(u8)]
pub enum PinMode {
    /// Floating input
    InputFloating,
    /// Pulled-up input
    InputPullup,
    /// Digital output.
    Output,
}

/// Interrupt input trigger modes that every pin configured as `Input` supports.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InterruptMode {
    /// Interrupts are disabled.
    None,
    /// Interrupts are raised when the input is `LOW` and so will typically
    /// happen on the `LOW` to `HIGH` transition. If interrupts are
    /// re-enabled while the input remains `HIGH`, a new interrupt will be raised
    /// without another transition being necessary.
    ActiveHigh,
    /// Interrupts are raised when the input is `LOW` and so will typically
    /// happen on the `HIGH` to `LOW` transition. If interrupts are
    /// re-enabled while the input remains `Low`, a new interrupt will be raised
    /// without another transition being necessary.
    ActiveLow,
    /// Interrupts are enabled on both the `HIGH` to `LOW` transition
    /// and the `LOW` to `HIGH` transition. If interrupts are
    /// re-enabled while the input remains in the state that triggered the interrupt, a
    /// new interrupt will _not_ be raised until another transition to the opposite
    /// state occurs.
    BothEdges,
}

/// A structure that represents an instance of the MCP23S17 I/O expander chip.
///
/// This is the key entrypoint into the driver. The user instantiates an `Mcp23s17` and
/// then uses [`Mcp23s17::pin_mode()`] to configure one GPIO `Pin` or
/// [`Mcp23s17::pin_mode_all()`] to configure all GPIO `Pin`s at once
/// Then, [`Mcp23s17::read()`] or [`Mcp23s17::set_high()`] / [`Mcp23s17::set_low()`] / [`Mcp23s17::set_value()`]
/// can be used to read from or control its pins
///
/// ```no_run
/// let mut mcp23s17 = mcp23s17::Mcp23s17::new(spi, cs_pin, 0b000).ok().unwrap();
///
/// // Configure pin GPA0 as an output
/// mcp23s17.pin_mode(0, mcp23s17::PinMode::Output).ok().unwrap();
/// // Set pin GPA0 high
/// mcp23s17.set_high(0);
///
/// // Configure pin GPA1 as an pullup input
/// mcp23s17.pin_mode(1, mcp23s17::PinMode::InputPullup).ok().unwrap();
/// // Read GPA1's level
/// let is_high = mcp23s17.read(1);
/// ```
pub struct Mcp23s17<
    SPI: embedded_hal::spi::SpiDevice
> {
    spi: SPI,

    /// The control byte to use in a message.
    ///
    /// The client address contains four fixed bits and three user-defined hardware
    /// address bits (if enabled via `IOCON::HAEN`) (pins A2, A1 and A0) with the
    /// read/write bit filling out the control byte.
    spi_read_control_byte: u8,
    spi_write_control_byte: u8,

    pin_io_configurations_a: u8,
    pin_io_configurations_b: u8,

    pin_interrupt_configurations_a: u8,
    pin_interrupt_configurations_b: u8,
}

#[inline]
fn pin_mask(pin_num: u8) -> u8 {
    0b1 << pin_num
}

impl<SPI: embedded_hal::spi::SpiDevice> Mcp23s17<SPI>
{
    /// Create an MCP23S17 instance
    #[allow(clippy::identity_op)]
    pub fn new(spi: SPI, address: u8) -> Result<Self> {

        let mut mcp = Mcp23s17 {
            spi,
            spi_read_control_byte: 0b01000000_u8 | 0b000 << 1 | 1 << 0,
            spi_write_control_byte: 0b01000000_u8 | 0b000 << 1 | 0 << 0,
            pin_io_configurations_a: 0,
            pin_io_configurations_b: 0,
            pin_interrupt_configurations_a: 0,
            pin_interrupt_configurations_b: 0,
        };
        // We enable HAEN on all connected devices so we can address them individually
        let iocon = mcp.read_byte(RegisterAddress::IOCON)?;
        mcp.write_byte(RegisterAddress::IOCON, iocon | IOCON::HAEN.bits())?;

        mcp.spi_read_control_byte = 0b01000000_u8 | address << 1 | 1 << 0;
        mcp.spi_write_control_byte = 0b01000000_u8 | address << 1 | 0 << 0;

        Ok(mcp)
    }

    /// Configure the pin with a given [PinMode]
    pub fn pin_mode(&mut self, pin: u8, mode: PinMode) -> Result<()> {
        let pin_num = pin % 8;
        let mask = pin_mask(pin_num);
        let (data_direction_register, pullup_register, has_interrupt) = if pin < 8 {
            (
                RegisterAddress::IODIRA,
                RegisterAddress::GPPUA,
                self.pin_interrupt_configurations_a & mask > 0,
            )
        } else {
            (
                RegisterAddress::IODIRB,
                RegisterAddress::GPPUB,
                self.pin_interrupt_configurations_b & mask > 0,
            )
        };

        // Configure Pullup
        match mode {
            PinMode::InputFloating | PinMode::Output => {
                self.clear_bit(pullup_register, pin_num)?;
            }
            PinMode::InputPullup => {
                self.set_bit(pullup_register, pin_num)?;
            }
        }

        // Configure Data Direction
        match mode {
            PinMode::InputFloating | PinMode::InputPullup => {
                self.set_bit(data_direction_register, pin_num)?;
                if has_interrupt {
                    self.set_interrupt_mode(pin, InterruptMode::None)?;
                }
            }
            PinMode::Output => {
                self.clear_bit(data_direction_register, pin_num)?;
                if pin < 8 {
                    self.clear_bit(RegisterAddress::GPIOA, pin_num)?;
                } else {
                    self.clear_bit(RegisterAddress::GPIOB, pin_num)?;
                }
            }
        }

        // Save configured Data Direction internally
        let configuration = if pin < 8 {
            &mut self.pin_io_configurations_a
        } else {
            &mut self.pin_io_configurations_b
        };

        match mode {
            PinMode::InputFloating | PinMode::InputPullup => {
                *configuration |= 1 << pin_num;
            }
            PinMode::Output => {
                *configuration &= !(1 << pin_num);
            }
        }

        Ok(())
    }

    /// Configure all of the pins to be a given [PinMode] at once
    pub fn pin_mode_all(&mut self, mode: PinMode) -> Result<()> {
        match mode {
            PinMode::InputFloating => {
                self.write_2_bytes(RegisterAddress::GPPUA, (0x00, 0x00))?;
                self.write_2_bytes(RegisterAddress::IODIRA, (0xff, 0xff))?;
                (self.pin_io_configurations_a, self.pin_io_configurations_b) = (0xff, 0xff);
            }
            PinMode::InputPullup => {
                self.write_2_bytes(RegisterAddress::GPPUA, (0xff, 0xff))?;
                self.write_2_bytes(RegisterAddress::IODIRA, (0xff, 0xff))?;
                (self.pin_io_configurations_a, self.pin_io_configurations_b) = (0xff, 0xff);
            }
            PinMode::Output => {
                self.write_2_bytes(RegisterAddress::GPPUA, (0x00, 0x00))?;
                self.write_2_bytes(RegisterAddress::IODIRA, (0x00, 0x00))?;
                (self.pin_io_configurations_a, self.pin_io_configurations_b) = (0x00, 0x00);
            }
        }
        Ok(())
    }

    /// Set all of the pins high.
    /// If they were not [PinMode::Output] before, they will be reconfigured
    pub fn set_all_high(&mut self) -> Result<()> {
        self.set_all_value((0xff, 0xff))
    }

    /// Set all of the pins low.
    /// If they were not [PinMode::Output] before, they will be reconfigured
    pub fn set_all_low(&mut self) -> Result<()> {
        self.set_all_value((0x00, 0x00))
    }

    /// Set all of the pins at once. The `value` is (PORT_A, PORT_B)
    /// If they were not [PinMode::Output] before, they will be reconfigured
    pub fn set_all_value(&mut self, value: (u8, u8)) -> Result<()> {
        if self.pin_io_configurations_a != 0x00 || self.pin_io_configurations_b != 0x00 {
            self.pin_mode_all(PinMode::Output)?;
        }
        self.write_2_bytes(RegisterAddress::GPIOA, value)
    }

    fn get_num_mask_register_config(&self, pin: u8) -> (u8, RegisterAddress, bool) {
        let pin_num = pin % 8;
        let mask = 0b1 << (pin % 8);
        if pin < 8 {
            (
                pin_num,
                RegisterAddress::GPIOA,
                self.pin_io_configurations_a & mask > 0,
            )
        } else {
            (
                pin_num,
                RegisterAddress::GPIOB,
                self.pin_io_configurations_b & mask > 0,
            )
        }
    }

    /// Set the specific pin high.
    /// If it was not [PinMode::Output] before, it will be reconfigured
    pub fn set_high(&mut self, pin: u8) -> Result<()> {
        let (pin_num, gpio_register, is_input) = self.get_num_mask_register_config(pin);
        if is_input {
            self.pin_mode(pin, PinMode::Output)?;
        }
        self.set_bit(gpio_register, pin_num)
    }

    /// Set the specific pin low.
    /// If it was not [PinMode::Output] before, it will be reconfigured
    pub fn set_low(&mut self, pin: u8) -> Result<()> {
        let (pin_num, gpio_register, is_input) = self.get_num_mask_register_config(pin);
        if is_input {
            self.pin_mode(pin, PinMode::Output)?;
        }
        self.clear_bit(gpio_register, pin_num)
    }

    /// Set the specific to the given [bool] `(true == HIGH)`.
    /// If it was not [PinMode::Output] before, it will be reconfigured
    pub fn set_value(&mut self, pin: u8, value: bool) -> Result<()> {
        if value {
            self.set_high(pin)
        } else {
            self.set_low(pin)
        }
    }

    /// Returns (PORT_A, PORT_B).
    /// Note that this will not reconfigure anything, as reading from a [PinMode::Output] is valid
    pub fn read_all(&mut self) -> Result<(u8, u8)> {
        self.read_2_bytes(RegisterAddress::GPIOA)
    }

    /// Returns the digital value present at the pin as a [bool] `(true == HIGH)`.
    /// Note that this will reconfigure the pin, even if it is a [PinMode::Output],
    /// as reading from a [PinMode::Output] is valid
    pub fn read(&mut self, pin: u8) -> Result<bool> {
        let gpio_register = if pin < 8 {
            RegisterAddress::GPIOA
        } else {
            RegisterAddress::GPIOB
        };
        self.get_bit(gpio_register, pin % 8)
    }

    /// Set the `Pin` to the requested [`InterruptMode`] (_i.e._ which edge(s) on the input
    /// trigger an interrupt.)
    ///
    /// Note that setting an `mode` of [`InterruptMode::None`] disables
    /// interrupts.
    ///
    /// The relevant register bits are set according to the following table:
    ///
    /// | Mode                           | `GPINTEN` | `INTCON` | `DEFVAL` |
    /// |--------------------------------|:---------:|:--------:|:--------:|
    /// | [`InterruptMode::None`]        |    `L`    |    `X`   |   `X`    |
    /// | [`InterruptMode::ActiveHigh`]  |    `H`    |    `H`   |   `L`    |
    /// | [`InterruptMode::ActiveLow`]   |    `H`    |    `H`   |   `H`    |
    /// | [`InterruptMode::BothEdges`]   |    `H`    |    `L`   |   `X`    |
    ///
    /// `X` = "Don't care" so register unchanged when setting this mode.

    pub fn set_interrupt_mode(&mut self, pin: u8, mode: InterruptMode) -> Result<()> {
        let pin_num = pin % 8;
        let mask = 0b1 << (pin % 8);
        let (gpinten, intcon, defval, is_input) = if pin < 8 {
            (
                RegisterAddress::GPINTENA,
                RegisterAddress::INTCONA,
                RegisterAddress::DEFVALA,
                self.pin_io_configurations_a & mask > 0,
            )
        } else {
            (
                RegisterAddress::GPINTENB,
                RegisterAddress::INTCONB,
                RegisterAddress::DEFVALB,
                self.pin_io_configurations_b & mask > 0,
            )
        };

        if !is_input {
            self.pin_mode(pin, PinMode::InputPullup)?;
        }

        // Set up the registers. Note that GPINTEN is set last so that the correct
        // criteria are set before enabling interrupts to avoid a spurious initial
        // interrupt.
        match mode {
            InterruptMode::None => {
                self.clear_bit(gpinten, pin_num)?;
                self.set_pin_interrupt_disabled(pin);
            }
            InterruptMode::ActiveHigh => {
                self.set_bit(intcon, pin_num)?;
                self.clear_bit(defval, pin_num)?;
                self.set_bit(gpinten, pin_num)?;
                self.set_pin_interrupt_enabled(pin);
            }
            InterruptMode::ActiveLow => {
                self.set_bit(intcon, pin_num)?;
                self.set_bit(defval, pin_num)?;
                self.set_bit(gpinten, pin_num)?;
                self.set_pin_interrupt_enabled(pin);
            }
            InterruptMode::BothEdges => {
                self.clear_bit(intcon, pin_num)?;
                self.set_bit(gpinten, pin_num)?;
                self.set_pin_interrupt_enabled(pin);
            }
        }
        Ok(())
    }
}

impl<SPI: embedded_hal::spi::SpiDevice> Mcp23s17<SPI>
{
    fn set_pin_interrupt_enabled(&mut self, pin: u8) {
        let pin_num = pin % 8;
        let mask = pin_mask(pin_num);
        if pin < 8 {
            self.pin_interrupt_configurations_a |= mask;
        } else {
            self.pin_interrupt_configurations_b |= mask;
        }
    }

    fn set_pin_interrupt_disabled(&mut self, pin: u8) {
        let pin_num = pin % 8;
        let mask = pin_mask(pin_num);
        if pin < 8 {
            self.pin_interrupt_configurations_a &= !mask;
        } else {
            self.pin_interrupt_configurations_b &= !mask;
        }
    }

    /// Get the specified bit in the register.
    ///
    /// Gets the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ANDing with a mask with the appropriate bit set.
    fn get_bit(&mut self, register: RegisterAddress, bit: u8) -> Result<bool> {
        Ok(self.read_byte(register)? & (0x01 << bit) > 0)
    }

    /// Set the specified bits in the register.
    ///
    /// Sets the bits by first reading the MCP23S17 register at `register` and then ORing
    /// it with `data` before writing it back to `register`.
    fn set_bits(&mut self, register: RegisterAddress, data: u8) -> Result<()> {
        let data = self.read_byte(register)? | data;
        self.write_byte(register, data)
    }

    /// Set the specified bit in the register.
    ///
    /// Sets the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ORing with a mask with the appropriate bit set before
    /// writing it back to `register`.
    fn set_bit(&mut self, register: RegisterAddress, bit: u8) -> Result<()> {
        self.set_bits(register, 0x01 << bit)
    }

    /// Clear the specified bits in the register.
    ///
    /// Clears the bits by first reading the MCP23S17 register at `register` and then ANDing
    /// it with `!data` before writing it back to `register`.
    fn clear_bits(&mut self, register: RegisterAddress, data: u8) -> Result<()> {
        let data = self.read_byte(register)? & !data;
        self.write_byte(register, data)
    }

    /// Clear the specified bit in the register.
    ///
    /// Clears the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ANDing with a mask with the appropriate bit cleared before
    /// writing it back to `register`.
    fn clear_bit(&mut self, register: RegisterAddress, bit: u8) -> Result<()> {
        self.clear_bits(register, 0x01 << bit)
    }

    /// Read a byte from the MCP23S17 register at the address `register`.
    fn read_byte(&mut self, register: RegisterAddress) -> Result<u8> {
        let mut write_buffer = [0u8; 3];
        write_buffer[0] = self.spi_read_control_byte;
        write_buffer[1] = register as u8;

        let read_buffer = self._transfer(&mut write_buffer)?;

        if read_buffer.len() != 3 {
            Err(Mcp23s17SpiError {})
        } else {
            Ok(read_buffer[2])
        }
    }

    /// Read a byte from the MCP23S17 register at the address `register`.
    fn read_2_bytes(&mut self, register: RegisterAddress) -> Result<(u8, u8)> {
        let mut write_buffer = [self.spi_read_control_byte, register as u8, 0, 0];

        let read_buffer = self._transfer(&mut write_buffer)?;

        Ok((read_buffer[2], read_buffer[3]))
    }

    fn _transfer<'a>(&mut self, write_buffer: &'a mut [u8]) -> Result<&'a [u8]> {
        self.spi
            .transfer_in_place(write_buffer)
            .map_err(|_| Mcp23s17SpiError {})?;
        Ok(write_buffer)
    }

    /// Write the byte `data` to the MCP23S17 register at address `register`.
    fn write_byte(&mut self, register: RegisterAddress, data: u8) -> Result<()> {
        let write_buffer = [self.spi_write_control_byte, register as u8, data];
        self._write(&write_buffer)
    }

    /// Write the 2 bytes `data` to the MCP23S17 register at address `register` and `register + 1`.
    fn write_2_bytes(&mut self, register: RegisterAddress, data: (u8, u8)) -> Result<()> {
        // `LOW` byte first
        let write_buffer = [self.spi_write_control_byte, register as u8, data.0, data.1];
        self._write(&write_buffer)
    }

    fn _write(&mut self, write_buffer: &[u8]) -> Result<()> {
        self.spi
            .write(write_buffer)
            .map_err(|_| Mcp23s17SpiError {})?;
        Ok(())
    }
}
