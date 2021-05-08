use embedded_hal::serial::{Read, Write};
use k64::{UART0, UART1, UART2, UART3, UART4, UART5};
use crate::Never;

#[derive(Debug)]
pub enum Error {
    Framing,
    Noise,
    Overrun,
    Parity,
    #[doc(hidden)]
    _Extensible,
}

pub enum Event {
    Rxne,
    Txe,
    Idle,
}

pub mod config {
    use crate::time::Hertz;

    pub enum WordLength {
        DataBits8,
        DataBits9,
    }

    pub enum Parity {
        ParityNone,
        ParityEven,
        ParityOdd,
    }

    pub enum StopBits {
        Stop1,
        Stop0p5,
        Stop2,
        Stop1p5
    }

    pub struct Config {
        pub baudrate: Hertz,
        pub wordlength: WordLength,
        pub parity: Parity,
        pub stopbits: StopBits
    }

    impl Config {
        pub fn baudrate(mut self, baudrate: impl Into<Hertz>) -> Self {
            self.baudrate = baudrate.into();
            self
        }

        pub fn parity_none(mut self) -> Self {
            self.parity = Parity::ParityNone;
            self
        }

        pub fn parity_even(mut self) -> Self {
            self.parity = Parity::ParityEven;
            self
        }

        pub fn parity_odd(mut self) -> Self {
            self.parity = Parity::ParityOdd;
            self
        }

        pub fn wordlength_8(mut self) -> Self {
            self.wordlength = WordLength::DataBits8;
            self
        }

        pub fn wordlength_9(mut self) -> Self {
            self.wordlength = WordLength::DataBits9;
            self
        }

        pub fn stopbits(mut self, stopbits: StopBits) -> Self {
            self.stopbits = stopbits;
            self
        }
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                baudrate: Hertz(19_200),
                wordlength: WordLength::DataBits8,
                parity: Parity::ParityNone,
                stopbits: StopBits::Stop1
            }
        }
    }

    impl<T: Into<Hertz>> From<T> for Config {
        fn from(f: T) -> Config {
            Config { 
                baudrate: f.into(),
                ..Default::default()
            }
        }
    }
}

