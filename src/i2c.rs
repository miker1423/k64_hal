use crate::pac::I2C0;
use crate::pac::{i2c0, sim};
use core::ops::Deref;
use crate::gpio::*;
use embedded_hal::blocking::i2c::{Write};

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

i2c_pins! {
    I2C0 => {
        scl => porte::PE24<AlternativeOD<AF5>>,
        sda => porte::PE25<AlternativeOD<AF5>>,
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum I2cError {
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
    ($I2C: ident, $scgcx: ident, $i2c: ident) => {
        impl private::Sealed for $I2C {}
        impl Instance for $I2C {
            unsafe fn enable_clock(sim: &sim::RegisterBlock) {
                sim.$scgcx.modify(|_, w| w.$i2c().set_bit());
            }
        }
    }
}

i2c!(I2C0, scgc4, i2c0);

impl<I2C, PINS> I2c<I2C, PINS>
where
    I2C: Instance
{
    pub fn new(i2c: I2C, pins: PINS, speed: u32, sim: &sim::RegisterBlock) -> Self
    where
        PINS: Pins<I2C>
    {
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
        self.i2c.s.modify(|_, w| w.arbl().set_bit().iicif().set_bit());
        self.i2c.c2.reset();
        self.i2c.flt.modify(|_, w| w.startf().set_bit().stopf().set_bit());
        self.i2c.ra.reset();

        self.i2c.c1.modify(|_, w| w.iicen().clear_bit());

        let _ = self.check_and_clear_error_flags();

        self.set_baudrate(speed);
        self.i2c.c1.modify(|_, w| w.iicen().set_bit());
    }

    fn set_baudrate(&self, _speed: u32){
        self.i2c.f.modify(|_, w| unsafe { w.icr().bits(44) });
    }

    fn check_and_clear_error_flags(&self) -> Result<i2c0::s::R, I2cError> {
        let status = self.i2c.s.read();
        let flt = self.i2c.flt.read();

        if status.arbl().bit_is_set() {
            self.i2c.s.modify(|_, w| w.arbl().set_bit());
            return Err(I2cError::ARBITRATION);
        }
        if status.rxak().bit_is_set() {
            return Err(I2cError::NACK);
        }
        if flt.startf().bit_is_set() {
            self.i2c.flt.modify(|_, w| w.startf().set_bit());
        }
        if flt.stopf().bit_is_set() {
            self.i2c.flt.modify(|_, w| w.stopf().set_bit());
        }

        Ok(status)
    }
}

trait I2cCommon {
    type Error;

    fn start_sequence(&self, address: u8) -> Result<(), Self::Error>;

    fn stop_sequence(&self) -> Result<(), Self::Error>;

    fn write_bytes(&self, address: u8, bytes: &[u8]) -> Result<(), Self::Error>;

    fn send_byte(&self, byte: u8) -> Result<(), Self::Error>;

    fn recv_byte(&self) -> Result<u8, Self::Error>;
}

impl<I2C, PINS> I2cCommon for I2c<I2C, PINS>
where
    I2C: Instance,
{
    type Error = I2cError;


    fn start_sequence(&self, address: u8) -> Result<(), Self::Error> {
        self.check_and_clear_error_flags()?;
        while {
          self.i2c.s.read().tcf().bit_is_clear()
        }{}
        self.check_and_clear_error_flags()?;
        self.i2c.c1.modify(|_, w| w.mst().set_bit().tx().set_bit());
        self.i2c.d.modify(|_, w| unsafe { w.bits((address << 1) | 0)});
        Ok(())
    }

    fn stop_sequence(&self) -> Result<(), Self::Error> {
        self.check_and_clear_error_flags()?;
        self.i2c.c1.modify(|_, w| w.mst().clear_bit()
                                            .tx().clear_bit()
                                            .txak().clear_bit());

        while {
            self.check_and_clear_error_flags()?.busy().bit_is_set()
        }{}

        Ok(())
    }

    fn write_bytes(&self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.start_sequence(address);
        if let Err(err) = self.check_and_clear_error_flags() {
            self.stop_sequence();
            return Err(err)
        }

        while {
            self.i2c.s.read().iicif().bit_is_clear()
        }{}

        for b in bytes {
            self.send_byte(*b);
        }

        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Self::Error> {
        while {
            let s = self.i2c.s.read();
            self.check_and_clear_error_flags()?.tcf().bit_is_clear()
        }{}
        self.i2c.s.modify(|_, w| w.iicif().set_bit());
        self.i2c.c1.modify(|_, w| w.tx().set_bit());
        let s = self.i2c.s.read();
        self.i2c.d.modify(|_, w| unsafe { w.data().bits(byte) });
        let s = self.i2c.s.read();
        while {
            self.check_and_clear_error_flags()?.iicif().bit_is_clear()
        }{}
        self.i2c.s.modify(|_, w| w.iicif().set_bit());

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Self::Error> {
        Ok(0)
    }
}

impl<I2C, PINS> Write for I2c<I2C, PINS>
where
    I2C: Instance
{
    type Error = I2cError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_bytes(address, bytes)?;
        self.stop_sequence()
    }
}