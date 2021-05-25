use core::{marker::PhantomData, convert::Infallible};
use embedded_hal::digital::v2::{
    OutputPin,
    InputPin,
    StatefulOutputPin,
    toggleable
};

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
    ($gpiox:ident) => {
        impl GpioRegExt for crate::pac::$gpiox::RegisterBlock {
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

gpio_trait!(gpiob);

macro_rules! gpio {
    ([$($PORTX:ident, $portx:ident, $GPIOX:ident, $PXx:ident => [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $pcri:ident),)+
    ]),+]) => {
        $(
            pub mod $portx {
                use core::{marker::PhantomData, convert::Infallible};
                use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, toggleable};
                use crate::pac::{$PORTX, $GPIOX, SIM};
                use cortex_m::interrupt::CriticalSection;

                use super::{
                    Alternative, GpioExt, Input, OpenDrain, Output, Floating, PullUp, PullDown,
                    AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7,
                    Pin, GpioRegExt,
                };

                pub struct Parts{
                    $(
                        pub $pxi: $PXi<$MODE>,
                    )+
                }

                impl GpioExt for $PORTX {
                    type Parts = Parts;

                    fn split(self) -> Parts {
                        unsafe { (&*SIM::ptr()) }.scgc5.modify(|_, w| w.$portx().set_bit());
                        Parts {
                            $(
                              $pxi: $PXi { _mode: PhantomData },
                            )+
                        }
                    }
                }

                $(
                    pub struct $PXi<MODE> {
                        _mode: PhantomData<MODE>,
                    }

                    impl<MODE> $PXi<MODE> {
                        pub fn enable(self, _cs: CriticalSection) -> Self {
                            Self::_set_alternate_function($i, 1);
                            self
                        }

                        pub fn disable(self, _cs: CriticalSection) -> Self {
                            Self::_set_alternate_function($i, 0);
                            self
                        }

                        fn _set_alternate_function(index: usize, mode: u8) {
                            unsafe { (&*$PORTX::ptr()) }.$pcri.write(|w| w.mux().bits(mode));
                        }

                        pub fn into_alternate_af0(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF0>> {
                            Self::_set_alternate_function($i, 0);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af1(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF1>> {
                            Self::_set_alternate_function($i, 1);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af2(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF2>> {
                            Self::_set_alternate_function($i, 2);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af3(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF3>> {
                            Self::_set_alternate_function($i, 3);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af4(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF4>> {
                            Self::_set_alternate_function($i, 4);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af5(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF5>> {
                            Self::_set_alternate_function($i, 5);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af6(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF6>> {
                            Self::_set_alternate_function($i, 6);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af7(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF7>> {
                            Self::_set_alternate_function($i, 7);
                            $PXi { _mode: PhantomData }
                        }
                    }


                    impl<MODE> $PXi<Output<MODE>> {
                        pub fn downgrade(self) -> Pin<Output<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
                                _mode: self._mode
                            }
                        }
                    }

                    impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                        fn is_set_high(&self) -> Result<bool, Self::Error> {
                            self.is_set_low().map(|v| !v)
                        }

                        fn is_set_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_set_low($i)})
                        }
                    }

                    impl<MODE> OutputPin for $PXi<Output<MODE>> {
                        type Error = Infallible;

                        fn set_high(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_high($i) })
                        }

                        fn set_low(&mut self) -> Result<(), Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).set_low($i) })
                        }
                    }

                    impl<MODE> toggleable::Default for $PXi<Output<MODE>> {}

                    impl InputPin for $PXi<Output<OpenDrain>> {
                        type Error = Infallible;

                        fn is_high(&self) -> Result<bool, Self::Error> {
                            self.is_low().map(|v| !v)
                        }

                        fn is_low(&self) -> Result<bool, Self::Error> {
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }

                    impl<MODE> $PXi<Input<MODE>> {
                        pub fn downgrade(self) -> Pin<Input<MODE>> {
                            Pin {
                                i: $i,
                                port: $GPIOX::ptr() as *const dyn GpioRegExt,
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
                            Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                        }
                    }
                )+
            }
        )+
    }
}

gpio!([
    PORTB, portb, GPIOB, PB => [
        PB0: (pb0, 0, Input<Floating>, pcr0),
        PB1: (pb1, 1, Input<Floating>, pcr1),
        PB2: (pb2, 2, Input<Floating>, pcr2),
        PB3: (pb3, 3, Input<Floating>, pcr3),
        PB4: (pb4, 4, Input<Floating>, pcr4),
        PB5: (pb5, 5, Input<Floating>, pcr5),
        PB6: (pb6, 6, Input<Floating>, pcr6),
        PB7: (pb7, 7, Input<Floating>, pcr7),
        PB8: (pb8, 8, Input<Floating>, pcr8),
        PB9: (pb9, 9, Input<Floating>, pcr9),
        PB10: (pb10, 10, Input<Floating>, pcr10),
        PB11: (pb11, 11, Input<Floating>, pcr11),
        PB12: (pb12, 12, Input<Floating>, pcr12),
        PB13: (pb13, 13, Input<Floating>, pcr13),
        PB14: (pb14, 14, Input<Floating>, pcr14),
        PB15: (pb15, 15, Input<Floating>, pcr15),
        PB16: (pb16, 16, Input<Floating>, pcr16),
        PB17: (pb17, 17, Input<Floating>, pcr17),
        PB18: (pb18, 18, Input<Floating>, pcr18),
        PB19: (pb19, 19, Input<Floating>, pcr19),
        PB20: (pb20, 20, Input<Floating>, pcr20),
        PB21: (pb21, 21, Input<Floating>, pcr21),
        PB22: (pb22, 22, Input<Floating>, pcr22),
        PB23: (pb23, 23, Input<Floating>, pcr23),
        PB24: (pb24, 24, Input<Floating>, pcr24),
        PB25: (pb25, 25, Input<Floating>, pcr25),
        PB26: (pb26, 26, Input<Floating>, pcr26),
        PB27: (pb27, 27, Input<Floating>, pcr27),
        PB28: (pb28, 28, Input<Floating>, pcr28),
        PB29: (pb29, 29, Input<Floating>, pcr29),
        PB30: (pb30, 30, Input<Floating>, pcr30),
        PB31: (pb31, 31, Input<Floating>, pcr31),
    ]
]);