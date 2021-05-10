use core::{marker::PhantomData, convert::Infallible};
use embedded_hal::digital::v2::{
    OutputPin,
    InputPin,
    StatefulOutputPin,
    toggleable
};
use k64::{GPIOA, SIM};

pub trait GpioExt {
    type Parts;

    fn split(self) -> Self::Parts;
}

trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;
pub struct AF6;
pub struct AF7;

pub struct Floating;
pub struct PullDown;
pub struct PullUp;
pub struct PushPull;
pub struct OpenDrain;

pub struct Input<MODE> {
    _mode: PhantomData<MODE>
}

pub struct Output<MODE> {
    _mode: PhantomData<MODE>
}

pub struct Alternative<AF> {
    _mode: PhantomData<AF>,
}

pub struct Pin<MODE> {
    i: u8,
    port: *const dyn GpioRegExt,
    _mode: PhantomData<MODE>,
}

unsafe impl<MODE> Sync for Pin<MODE> {}
unsafe impl<MODE> Send for Pin<MODE> {}

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_set_low(self.i) })
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_low(self.i) };
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_high(self.i) };
        Ok(())
    }
}

impl<MODE> toggleable::Default for Pin<Output<MODE>> {}


impl InputPin for Pin<Output<OpenDrain>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

macro_rules! gpio_trait {
    ($gpio:ident) => {
        impl GpioRegExt for crate::pac::$gpio::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                (self.pdir.read().bits() >> pos) == 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                (self.pdir.read().bits() >> pos) == 0
            }

            fn set_high(&self, pos: u8) {
                self.psor.write(|w| unsafe { w.bits(1 << pos) })
            }

            fn set_low(&self, pos: u8) {
                self.pcor.write(|w| unsafe { w.bits(1 << pos) })
            }
        }
    }
}

gpio_trait!(gpioa);

macro_rules!gpio {
    ([$($PORTX:ident, $portx:ident, $iopxenr:ident, $PXx:ident => [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]),+]) => {
        $(
            pub mod $portx {
                use core::{marker::PhandomData, convert::Infallible};
                use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, toggleable};
                use crate::pac::$GPIOX;
                use cortex_m::interrupt::CriticalSection;

                use super::{
                    Alternate, GpioExt, Input, OpenDrain, Output, Floating, PullUp, PullDown,
                    AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7
                    Pin, GpioRegExt,
                };

                pub struct Parts{
                    $(
                        pub $pxi: $PXi<$MODE>
                    )+
                }

                impl GpioExt for $PORTX {
                    type Parts = Parts;

                    fn split(self) -> Parts {
                        Parts {
                            $(
                              $pxi: $PXi { _mode: PhandomData }
                            )+
                        }
                    }
                }

                fn _set_alternate_function(index: usize, mode: u8) {
                    unsafe { (&*$PORTX::ptr()) }.pcr$i.write(|w| w.mux().bits(mode));
                }

                $(
                    pub struct $PXi<MODE> {
                        _mode: PhandomData<MODE>,
                    }

                    impl<MODE> for $PXi<MODE> {
                        pub fn enable(self, _cs: CriticalSection) -> Self {
                            _set_alternate_function($i, 1);
                            self
                        }

                        pub fn disable(self, _cs: CriticalSection) -> Self {
                            _set_alternate_function($i, 0);
                            self
                        }
                    }

                    pub fn into_alternate_af0(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF0>> {
                        _set_alternate_function($i, 0);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af1(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF1>> {
                        _set_alternate_function($i, 1);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af2(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF2>> {
                        _set_alternate_function($i, 2);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af3(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF3>> {
                        _set_alternate_function($i, 3);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af4(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF4>> {
                        _set_alternate_function($i, 4);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af5(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF5>> {
                        _set_alternate_function($i, 5);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af6(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF6>> {
                        _set_alternate_function($i, 6);
                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_alternate_af7(
                        self, _cs: &CriticalSection
                    ) -> $PXi<Alternate<AF7>> {
                        _set_alternate_function($i, 7);
                        $PXi { _mode: PhantomData }
                    }


                    impl<MODE> $PXi<Output<MODE>> {
                        pub fn downgrade(self) -> Pin<Output<MODE>> {
                            Pin {
                                i: $i,
                                port: $PORTX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode
                            }
                        }
                    }

                    impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                        fn is_set_high(&self) -> Result<bool, Self::Error> {
                            self.is_set_low().map(|v| !v)
                        }

                        fn is_set_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$PORTX::ptr()).is_set_low($i)})
                        }
                    }

                    impl<MODE> OutputPin for $PXi<Output<MODE>> {
                        type Error = Infallible;

                        fn set_high(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$PORTX::ptr()).set_high($i) })
                        }

                        fn set_low(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$PORTX::ptr()).set_low($i) })
                        }
                    }

                    impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                    impl InputPin for $PXi<Output<OpenDrain>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$PORTX::ptr()).is_low($i) })
                        }
                    }

                    impl<MODE> $PXi<Input<MODE>> {
                        pub fn downgrade(self) -> Pin<Input<MODE>> {
                            Pin {
                                i: $i,
                                port: $PORTX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode
                            }
                        }
                    }

                    impl<MODE> InputPin for $PXi<Input<MODE>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$PORTX::ptr()).is_low($i) })
                        }
                    }
                )+
            }
        )+
    }
}
