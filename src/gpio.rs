use core::marker::PhantomData;

use embedded_hal::digital::v2::{
    OutputPin
};
use k64::{gpioa, GPIOA, Peripherals};
use k64::{ sim, sim::SCGC5 };
use k64::ftm0::MOD;
use core::pin::Pin;

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

// Pin 0 de puerto A
pub struct PA2<MODE> {
    _mode: PhantomData<MODE>
}

impl<MODE> OutputPin for PT0<MODE> {
    type Error = PinError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let p = Peripherals::take();
        if let None = p {
            return Err(PinError);
        }

        let p = p.unwrap();
        p.GPIOA.pcor.write(|w| unsafe { w.bits(1 << 0) });

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let p = Peripherals::take();
        if let None = p {
            return Err(PinError);
        }

        let p = p.unwrap();
        p.GPIOA.psor.write(|w| unsafe { w.bits(1 << 0) });

        Ok(())
    }
}

// Puerto A
pub struct PTA<MODE> {
    i: i32,
    _mode: PhantomData<MODE>
}

impl<MODE> OutputPin for PTA<MODE> {
    type Error = PinError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let p = Peripherals::take();
        if let None = p {
            return Err(PinError);
        }

        let p = p.unwrap();
        p.GPIOA.pcor.write(|w| unsafe { w.bits(1 << self.i as u32) });
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let p = Peripherals::take();
        if let None = p {
            return Err(PinError);
        }

        let p = p.unwrap();
        p.GPIOA.psor.write(|w| unsafe { w.bits(1 << self.i as u32) });
        Ok(())
    }
}

impl GpioExt for GPIOA {
    type Parts = Parts;
    fn split(self) -> Option<Parts> {
        let p = k64::Peripherals::take();
        if let None = p {
            return None;
        }
        let p = p.unwrap();

        p.SIM.scgc5.modify(|_, w| w.porta().set_bit());

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