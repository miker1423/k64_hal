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

        self.i2c.a1.reset();
        self.i2c.f.reset();
        self.i2c.c1.reset();
        self.i2c.s.reset();
        self.i2c.c2.reset();

        self.i2c.flt.modify(|_, w| w.startf().set_bit().stopf().set_bit());
        self.i2c.s.modify(|_, w|
            w.arbl().set_bit()
                .iicif().set_bit()
        );

        self.i2c.c1.modify(|_, w| w.iicen().set_bit());
    }

    fn check_and_clear_error_flags(&self) -> Result<i2c0::s::R, Self::Error> {
        let status = self.i2c.s.read();

        if status.arbl().bit_is_set() {
            self.i2c.s.modify(|_, w| w.arbl().set_bit());
            return Err(Error::ARBITRATION);
        }
        if status.busy().bit_is_set() {
            return Err(Error::BUS);
        }
        if status.rxak().bit_is_set() {
            return Err(Error::NACK);
        }

        Ok(status)
    }
}

trait I2cCommon {
    type Error;
    fn write_bytes(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error>;

    fn send_byte(&self, byte: u8) -> Result<(), Self::Error>;

    fn recv_byte(&self) -> Result<u8, Self::Error>;
}

impl<I2C, PINS> I2cCommon for I2c<I2C, PINS>
where
    I2C: Instance,
{
    type Error = Error;
    fn write_bytes(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.check_and_clear_error_flags()?;
        self.i2c.c1.modify(|_, w| w.mst().set_bit().tx().set_bit());

        self.check_and_clear_error_flags()?.rxak().bit_is_set();

        while {
            self.check_and_clear_error_flags()?;
            let s = self.i2c.s.read();
            s.busy().bit_is_clear()
        } { }

        self.i2c.d.modify(|_, w| unsafe { w.data().bits(address << 1)  });

        while self.check_and_clear_error_flags()?.rxak().bit_is_set() {}
        self.check_and_clear_error_flags()?;
        for b in bytes {
            self.send_byte(*b);
        }

        self.i2c.c1.modify(|_, w| w.mst().clear_bit().tx().clear_bit().txak().clear_bit());
        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Self::Error> {
        while {
            self.check_and_clear_error_flags()?.tcf().bit_is_clear()
        }{}
        self.i2c.d.modify(|_, w| unsafe { w.data().bits(byte) });
        while {
            self.check_and_clear_error_flags()?.tcf().bit_is_clear()
        }{}

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Self::Error> {
        Ok(0)
    }
}