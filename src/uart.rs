use embedded_hal::serial::{Read, Write};
use core::{marker::PhantomData, convert::Infallible};
use crate::pac::SIM;
use crate::gpio::*;

#[derive(Debug)]
pub enum UartError {
    Framing,
    Noise,
    Overrun,
    Parity,
}

pub struct BaudRate(pub u32);

impl Into<BaudRate> for u32 {
    fn into(self) -> BaudRate {
        BaudRate(self)
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum WordLength {
    DataBits8,
    DataBits9,
}

#[derive(PartialOrd, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(PartialOrd, PartialEq)]
pub enum StopBits {
    Stop1,
    Stop2,
}

pub struct Config {
    baudrate: BaudRate,
    word_length: WordLength,
    parity: Parity,
    stop_bits: StopBits
}

pub trait RxPin<UART> { }
pub trait TxPin<UART> { }

macro_rules! uart_pins {
    ($($UART:ident => {
        tx => [$($tx:ty), + $(,)*],
        rx => [$($rx:ty), + $(,)*],
    })+) => {
        $(
            $(
                impl TxPin<crate::pac::$UART> for $tx { }
            )+
            $(
                impl RxPin<crate::pac::$UART> for $rx { }
            )+
        )+
    }
}

uart_pins! {
    UART0 => {
        tx => [portb::PB17<Alternative<AF3>>],
        rx => [portb::PB16<Alternative<AF3>>],
    }
}

pub struct Rx<UART> {
    _instance: PhantomData<UART>,
}

pub struct Tx<UART> {
    _instance: PhantomData<UART>
}

pub struct Serial<UART, TXPIN, RXPIN> {
    uart: UART,
    pins: (TXPIN, RXPIN),
}

impl<UART, TXPIN, RXPIN> Serial<UART, TXPIN, RXPIN>
{
    pub fn split(self) -> (Tx<UART>, Rx<UART>)
        where
            TXPIN: TxPin<UART>,
            RXPIN: RxPin<UART>
    {
        (
            Tx {
                _instance: PhantomData,
            },
            Rx {
                _instance: PhantomData,
            }
        )
    }

    pub fn relase(self) -> (TXPIN, RXPIN) {
        self.pins
    }
}

impl Config {
    pub fn new(baudrate: BaudRate, parity: Parity, word_length: WordLength, stop_bits: StopBits) -> Config {
        Config {baudrate, parity, word_length, stop_bits}
    }
}

trait ConfigMethod {
    fn configure(&self, config: &Config, sim: &SIM);

    fn get_real_baudrate(baudrate: &BaudRate) -> u32;
}

macro_rules! uart {
    ($($UART:ident: ($uart:ident, $uarttx:ident, $uartrx:ident, $scgc:ident),)+) => {
        $(
            use crate::pac::$UART;

            impl<TXPIN, RXPIN> Serial<$UART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$UART>,
                    RXPIN: RxPin<$UART>
            {
                pub fn $uart(uart: $UART, pins: (TXPIN, RXPIN), config: &Config, sim: &SIM) -> Self {
                    let serial = Serial { uart, pins };
                    serial.configure(config, sim);
                    serial
                }
            }

            impl<TXPIN> Serial<$UART, TXPIN, ()>
                where
                    TXPIN: TxPin<$UART>,
            {
                pub fn $uarttx(uart: $UART, txpin: TXPIN, config: &Config, sim: &SIM) -> Self {
                    let rxpin = ();
                    let serial = Serial { uart, pins: (txpin, rxpin) };
                    serial.configure(config, sim);
                    serial
                }
            }

            impl<RXPIN> Serial<$UART, (), RXPIN>
                where
                    RXPIN: RxPin<$UART>
            {
                pub fn $uartrx(uart: $UART, rxpin: RXPIN, config: &Config, sim: &SIM) -> Self {
                    let txpin = ();
                    let serial = Serial { uart, pins: (txpin, rxpin)};
                    serial.configure(config, sim);
                    serial
                }
            }

            impl core::fmt::Write for Tx<$UART>
                where
                    Tx<$UART>: embedded_hal::serial::Write<u8>,
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write(*c)))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl<TXPIN, RXPIN> core::fmt::Write for Serial<$UART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$UART>,
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    s.as_bytes()
                        .iter()
                        .try_for_each(|c| nb::block!(self.write(*c)))
                        .map_err(|_| core::fmt::Error)
                }
            }

            impl Read<u8> for Rx<$UART>
            {
                type Error = UartError;

                fn read(&mut self) -> nb::Result<u8, Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).s1.read() };
                    if status_register.or().bit() {
                        Err(nb::Error::Other(UartError::Overrun))
                    } else if status_register.fe().bit() {
                        Err(nb::Error::Other(UartError::Framing))
                    } else if status_register.nf().bit() {
                        Err(nb::Error::Other(UartError::Noise))
                    } else if status_register.pf().bit() {
                        Err(nb::Error::Other(UartError::Parity))
                    } else if status_register.rdrf().bit() {
                        let d = unsafe {  (&*$UART::ptr())}.d.read();
                        Ok(d.bits())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl<TXPIN, RXPIN> Read<u8> for Serial<$UART, TXPIN, RXPIN>
                where
                    RXPIN: RxPin<$UART>
            {
                type Error = UartError;

                fn read(&mut self) -> nb::Result<u8, Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()).s1.read() };
                    if status_register.or().bit() {
                        Err(nb::Error::Other(UartError::Overrun))
                    } else if status_register.fe().bit() {
                        Err(nb::Error::Other(UartError::Framing))
                    } else if status_register.nf().bit() {
                        Err(nb::Error::Other(UartError::Noise))
                    } else if status_register.pf().bit() {
                        Err(nb::Error::Other(UartError::Parity))
                    } else if status_register.rdrf().bit() {
                        let d = unsafe {  (&*$UART::ptr())}.d.read();
                        Ok(d.bits())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl Write<u8> for Tx<$UART> {
                type Error = Infallible;

                fn write(&mut self, data: u8) -> nb::Result<(), Self::Error>
                {
                    let uart = unsafe { (&*$UART::ptr())};
                    let status_register = uart.s1.read();
                    if status_register.tdre().bit() {
                        uart.d.write(|w| unsafe { w.bits(data) });
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()) }.s1.read();
                    if status_register.tc().bit() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl<TXPIN, RXPIN> Write<u8> for Serial<$UART, TXPIN, RXPIN>
                where
                    TXPIN: TxPin<$UART>
            {
                type Error = Infallible;

                fn write(&mut self, data: u8) -> nb::Result<(), Self::Error>
                {
                    let uart = unsafe { (&*$UART::ptr())};
                    let status_register = uart.s1.read();
                    if status_register.tdre().bit() {
                        uart.d.write(|w| unsafe { w.bits(data) });
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn flush(&mut self) -> nb::Result<(), Self::Error>
                {
                    let status_register = unsafe { (&*$UART::ptr()) }.s1.read();
                    if status_register.tc().bit() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl<TXPIN, RXPIN> ConfigMethod for Serial<$UART, TXPIN, RXPIN> {
                fn configure(&self, config: &Config, sim: &SIM) {
                    sim.$scgc.modify(|_, w| w.$uart().set_bit());
                    let uart = unsafe { (&*UART0::ptr())};
                    uart.c2.modify(|_, w| w.te().clear_bit().re().clear_bit());
                    let baudrate = Self::get_real_baudrate(&config.baudrate);
                    let baudrate_high = ((baudrate & 0x1F00) >> 8) as u8;
                    let baudrate_low = (baudrate & 0xFF) as u8;
                    uart.bdh.modify(|_, w| unsafe {
                        w.sbr().bits(baudrate_high)
                            .sbns().bit(config.stop_bits == StopBits::Stop2)
                    });
                    uart.bdl.modify(|_, w| unsafe { w.sbr().bits(baudrate_low) });
                    uart.c1.modify(|_, w| {
                        let is_nine_bit = config.word_length == WordLength::DataBits9;
                        w.pe().bit(config.parity != Parity::None)
                            .pt().bit(config.parity == Parity::Odd)
                            .m().bit(is_nine_bit)
                    });
                    uart.c2.modify(|_, w| w.te().set_bit().re().set_bit());
                }

                fn get_real_baudrate(baudrate: &BaudRate) -> u32 {
                    (20_971_520 / (baudrate.0 * 16)) + 1
                }
            }
        )+
    }
}

uart! {
    UART0: (uart0, uart0tx, uart0rx, scgc4),
}