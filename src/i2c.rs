use crate::pac::I2C0;
use crate::pac::{i2c0, sim};
use core::ops::Deref;
use crate::pac::SIM;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

pub struct I2c<I2C: Instance, PINS> {
    i2c: I2C,
    pins: PINS,
}

pub trait Pins<I2c> {}
pub trait PinScl<I2c> {}
pub trait PinSda<I2c> {}

impl<I2c, SCL, SDA> Pins<I2c> for (SCL, SDA)
where
    SCL: PinScl<I2c>,
    SDA: PinSda<I2c>
{
}

macro_rules! i2c_pins {
    ($($I2C:ident => {
        scl => $scl: ty,
        sda => $sda: ty,
    })+) => {
        $(
            impl PinScl<crate::pac::$I2C> for $scl {}
            impl PinSda<crate::pac::$I2C> for $sda {}
        )+
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    OVERRUN,
    NACK,
    TIMEOUT,
    BUS,
    CRC,
    ARBITRATION,
}

mod private {
    pub trait Sealed {}
}

pub trait Instance: private::Sealed + Deref<Target = i2c0::RegisterBlock> {
    #[doc(hidden)]
    unsafe fn enable_clock(sim: &sim::RegisterBlock);
}

macro_rules! i2c {
    ($($I2C: ident, $scgcx: ident, $i2c: ident)+) => {
        impl private::Sealed for $I2C {}
        impl Instance for $I2C for $I2C {
            unsafe fn enable_clock(sim: &sim::RegisterBlock) {
                sim.$scgcx.modify(|_, w| w.$i2c().set_bit());
            }
        }
    }
}

trait I2cCommon {
    fn write_bytes(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error>;

    fn send_byte(&self, byte: u8) -> Result<(), Error>;

    fn recv_byte(&self) -> Result<u8, Error>;
}

impl<I2C, PINS> I2c<I2C, PINS>
where
    I2C: Instance
{
    pub fn new(i2c: I2C, pins: PINS, speed: u32, sim: &sim::RegisterBlock) -> Self {
        unsafe { I2C::enable_clock(sim) };

        let i2c = I2c {i2c, pins};
        i2c.i2c_init(speed);
        i2c
    }

    fn i2c_init(&self, speed: u32) {
        self.i2c.c1.modify(|_, w| w.iicen().clear_bit());

        self.i2c.c1.modify(|_, w| w.iicen().set_bit());
    }
}