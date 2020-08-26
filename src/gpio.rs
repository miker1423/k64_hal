use core::marker::PhantomData;

use embedded_hal::digital::v2::{
    OutputPin,
    InputPin,
    StatefulOutputPin,
    toggleable
};
use k64::{GPIOA, SIM};
use crate::Never;

pub trait GpioExt {
    type Parts;

    fn split(self) -> Option<Self::Parts>;
}

pub struct Input<MODE> {
    _mode: PhantomData<MODE>
}

pub struct Output<MODE> {
    _mode: PhantomData<MODE>
}

pub struct Floating;
pub struct PullDown;
pub struct PullUp;
pub struct PushPull;
pub struct OpenDrain;
pub struct PinError;

pub struct Parts {
    pub pa0: PA2<Floating>
}

// Pin 2 de puerto A
pub struct PA2<MODE> {
    _mode: PhantomData<MODE>
}

impl<MODE> OutputPin for PA2<MODE> {
    type Error = Never;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*GPIOA::ptr()).pcor.write(|w| w.bits(1 << 2)); }
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*GPIOA::ptr()).psor.write(|w| w.bits(1 << 2)); }
        Ok(())
    }
}

impl<MODE> InputPin for PA2<MODE> {
    type Error = Never;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { ((*GPIOA::ptr()).pdir.read().bits() >> 2 as u32) == 0 })
    }
}

impl<MODE> StatefulOutputPin for PA2<MODE> {
    fn is_set_high(&self) -> Result<bool, Never> {
        self.is_set_low().map(|v| !v)
    }

    fn is_set_low(&self) -> Result<bool, Never>{
        Ok(unsafe { ((*GPIOA::ptr()).pdir.read().bits() >> 2 as u32) == 0 })
    }
}

impl<MODE> toggleable::Default for PA2<MODE> { }

// Puerto A
pub struct PTA<MODE> {
    i: i32,
    _mode: PhantomData<MODE>
}

impl<MODE> InputPin for PTA<MODE> {
    type Error = Never;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { ((*GPIOA::ptr()).pdir.read().bits() >> self.i as u32) == 0 })
    }
}

impl<MODE> OutputPin for PTA<MODE> {
    type Error = Never;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*GPIOA::ptr()).pcor.write(|w| w.bits(1 << self.i as u32)); }
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*GPIOA::ptr()).psor.write(|w| w.bits(1 << self.i as u32)); }
        Ok(())
    }
}
impl<MODE> StatefulOutputPin for PTA<MODE> {
    fn is_set_high(&self) -> Result<bool, Never> {
        self.is_set_low().map(|v| !v)
    }

    fn is_set_low(&self) -> Result<bool, Never>{
        Ok(unsafe { ((*GPIOA::ptr()).pdir.read().bits() >> self.i as u32) == 0 })
    }
}

impl<MODE> toggleable::Default for PTA<MODE> { }

impl GpioExt for GPIOA {
    type Parts = Parts;
    fn split(self) -> Option<Parts> {
        unsafe  { (*SIM::ptr()).scgc5.modify(|_, w| w.porta().set_bit()); }

        Some(Parts {
            pa0: PA2 { _mode: PhantomData }
        })
    }
}

/*
macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $gpioy:ident, $iopxenr:ident, $iopxrst:ident, $PXx:ident, [
        $($PXi:ident: ($pxi:ident, $i:ident, $MODE:ty, $AFR:ident),)+
    ]) => {
        pub mod $gpiox {
            use core::marker::PhantomData;

            use hal::digital::OutputPin;
            use k64::{$gpioy, $GPIOX};
        }
    };
}
*/