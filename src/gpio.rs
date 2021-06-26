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

pub struct AlternativeOD<MODE> {
    _mode: PhantomData<MODE>
}

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

gpio_trait!(gpioa);
gpio_trait!(gpiob);
gpio_trait!(gpioc);
gpio_trait!(gpiod);
gpio_trait!(gpioe);

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
                    Alternative, GpioExt, Input, OpenDrain, Output, Floating, AlternativeOD,
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
                        pub fn enable(self, _cs: &CriticalSection) -> Self {
                            Self::_set_alternate_function(1);
                            self
                        }

                        pub fn disable(self, _cs: &CriticalSection) -> Self {
                            Self::_set_alternate_function(0);
                            self
                        }

                        fn _set_alternate_function(mode: u8) {
                            unsafe { (&*$PORTX::ptr()) }.$pcri.modify(|_, w| w.mux().bits(mode));
                        }

                        pub fn into_alternate_af0(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF0>> {
                            Self::_set_alternate_function(0);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af1(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF1>> {
                            Self::_set_alternate_function(1);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af2(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF2>> {
                            Self::_set_alternate_function(2);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af3(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF3>> {
                            Self::_set_alternate_function(3);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af4(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF4>> {
                            Self::_set_alternate_function(4);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af5(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF5>> {
                            Self::_set_alternate_function(5);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af6(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF6>> {
                            Self::_set_alternate_function(6);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_alternate_af7(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Alternative<AF7>> {
                            Self::_set_alternate_function(7);
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af0_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF0>> {
                            Self::_set_alternate_function(0);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af1_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF1>> {
                            Self::_set_alternate_function(1);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af2_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF2>> {
                            Self::_set_alternate_function(2);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af3_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF3>> {
                            Self::_set_alternate_function(3);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af4_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF4>> {
                            Self::_set_alternate_function(4);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af5_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF5>> {
                            Self::_set_alternate_function(5);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_af6_outputdrain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<AlternativeOD<AF6>> {
                            Self::_set_alternate_function(6);
                            let port = unsafe { &(*$PORTX::ptr()) };
                            port.$pcri.modify(|_, w|
                                unsafe { w.bits(0x23) }
                            );
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_output(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<Floating>> {
                            let gpio = unsafe { &(*$GPIOX::ptr()) };
                            gpio.pddr.modify(|_, w| unsafe { w.bits(1 << $i) });
                            $PXi { _mode: PhantomData }
                        }

                        pub fn into_open_drain(
                            self, _cs: &CriticalSection
                        ) -> $PXi<Output<OpenDrain>> {

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
    PORTA, porta, GPIOA, PA => [
        PA0: (pa0, 0, Input<Floating>, pcr0),
        PA1: (pa1, 1, Input<Floating>, pcr1),
        PA2: (pa2, 2, Input<Floating>, pcr2),
        PA3: (pa3, 3, Input<Floating>, pcr3),
        PA4: (pa4, 4, Input<Floating>, pcr4),
        PA5: (pa5, 5, Input<Floating>, pcr5),
        PA6: (pa6, 6, Input<Floating>, pcr6),
        PA7: (pa7, 7, Input<Floating>, pcr7),
        PA8: (pa8, 8, Input<Floating>, pcr8),
        PA9: (pa9, 9, Input<Floating>, pcr9),
        PA10: (pa10, 10, Input<Floating>, pcr10),
        PA11: (pa11, 11, Input<Floating>, pcr11),
        PA12: (pa12, 12, Input<Floating>, pcr12),
        PA13: (pa13, 13, Input<Floating>, pcr13),
        PA14: (pa14, 14, Input<Floating>, pcr14),
        PA15: (pa15, 15, Input<Floating>, pcr15),
        PA16: (pa16, 16, Input<Floating>, pcr16),
        PA17: (pa17, 17, Input<Floating>, pcr17),
        PA18: (pa18, 18, Input<Floating>, pcr18),
        PA19: (pa19, 19, Input<Floating>, pcr19),
        PA20: (pa20, 20, Input<Floating>, pcr20),
        PA21: (pa21, 21, Input<Floating>, pcr21),
        PA22: (pa22, 22, Input<Floating>, pcr22),
        PA23: (pa23, 23, Input<Floating>, pcr23),
        PA24: (pa24, 24, Input<Floating>, pcr24),
        PA25: (pa25, 25, Input<Floating>, pcr25),
        PA26: (pa26, 26, Input<Floating>, pcr26),
        PA27: (pa27, 27, Input<Floating>, pcr27),
        PA28: (pa28, 28, Input<Floating>, pcr28),
        PA29: (pa29, 29, Input<Floating>, pcr29),
        PA30: (pa30, 30, Input<Floating>, pcr30),
        PA31: (pa31, 31, Input<Floating>, pcr31),
    ],
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
    ],
    PORTC, portc, GPIOC, PC => [
        PC0: (pc0, 0, Input<Floating>, pcr0),
        PC1: (pc1, 1, Input<Floating>, pcr1),
        PC2: (pc2, 2, Input<Floating>, pcr2),
        PC3: (pc3, 3, Input<Floating>, pcr3),
        PC4: (pc4, 4, Input<Floating>, pcr4),
        PC5: (pc5, 5, Input<Floating>, pcr5),
        PC6: (pc6, 6, Input<Floating>, pcr6),
        PC7: (pc7, 7, Input<Floating>, pcr7),
        PC8: (pc8, 8, Input<Floating>, pcr8),
        PC9: (pc9, 9, Input<Floating>, pcr9),
        PC10: (pc10, 10, Input<Floating>, pcr10),
        PC11: (pc11, 11, Input<Floating>, pcr11),
        PC12: (pc12, 12, Input<Floating>, pcr12),
        PC13: (pc13, 13, Input<Floating>, pcr13),
        PC14: (pc14, 14, Input<Floating>, pcr14),
        PC15: (pc15, 15, Input<Floating>, pcr15),
        PC16: (pc16, 16, Input<Floating>, pcr16),
        PC17: (pc17, 17, Input<Floating>, pcr17),
        PC18: (pc18, 18, Input<Floating>, pcr18),
        PC19: (pc19, 19, Input<Floating>, pcr19),
        PC20: (pc20, 20, Input<Floating>, pcr20),
        PC21: (pc21, 21, Input<Floating>, pcr21),
        PC22: (pc22, 22, Input<Floating>, pcr22),
        PC23: (pc23, 23, Input<Floating>, pcr23),
        PC24: (pc24, 24, Input<Floating>, pcr24),
        PC25: (pc25, 25, Input<Floating>, pcr25),
        PC26: (pc26, 26, Input<Floating>, pcr26),
        PC27: (pc27, 27, Input<Floating>, pcr27),
        PC28: (pc28, 28, Input<Floating>, pcr28),
        PC29: (pc29, 29, Input<Floating>, pcr29),
        PC30: (pc30, 30, Input<Floating>, pcr30),
        PC31: (pc31, 31, Input<Floating>, pcr31),
    ],
    PORTD, portd, GPIOD, PD => [
        PD0: (pd0, 0, Input<Floating>, pcr0),
        PD1: (pd1, 1, Input<Floating>, pcr1),
        PD2: (pd2, 2, Input<Floating>, pcr2),
        PD3: (pd3, 3, Input<Floating>, pcr3),
        PD4: (pd4, 4, Input<Floating>, pcr4),
        PD5: (pd5, 5, Input<Floating>, pcr5),
        PD6: (pd6, 6, Input<Floating>, pcr6),
        PD7: (pd7, 7, Input<Floating>, pcr7),
        PD8: (pd8, 8, Input<Floating>, pcr8),
        PD9: (pd9, 9, Input<Floating>, pcr9),
        PD10: (pd10, 10, Input<Floating>, pcr10),
        PD11: (pd11, 11, Input<Floating>, pcr11),
        PD12: (pd12, 12, Input<Floating>, pcr12),
        PD13: (pd13, 13, Input<Floating>, pcr13),
        PD14: (pd14, 14, Input<Floating>, pcr14),
        PD15: (pd15, 15, Input<Floating>, pcr15),
        PD16: (pd16, 16, Input<Floating>, pcr16),
        PD17: (pd17, 17, Input<Floating>, pcr17),
        PD18: (pd18, 18, Input<Floating>, pcr18),
        PD19: (pd19, 19, Input<Floating>, pcr19),
        PD20: (pd20, 20, Input<Floating>, pcr20),
        PD21: (pd21, 21, Input<Floating>, pcr21),
        PD22: (pd22, 22, Input<Floating>, pcr22),
        PD23: (pd23, 23, Input<Floating>, pcr23),
        PD24: (pd24, 24, Input<Floating>, pcr24),
        PD25: (pd25, 25, Input<Floating>, pcr25),
        PD26: (pd26, 26, Input<Floating>, pcr26),
        PD27: (pd27, 27, Input<Floating>, pcr27),
        PD28: (pd28, 28, Input<Floating>, pcr28),
        PD29: (pd29, 29, Input<Floating>, pcr29),
        PD30: (pd30, 30, Input<Floating>, pcr30),
        PD31: (pd31, 31, Input<Floating>, pcr31),
    ],
    PORTE, porte, GPIOE, PE => [
        PE0: (pe0, 0, Input<Floating>, pcr0),
        PE1: (pe1, 1, Input<Floating>, pcr1),
        PE2: (pe2, 2, Input<Floating>, pcr2),
        PE3: (pe3, 3, Input<Floating>, pcr3),
        PE4: (pe4, 4, Input<Floating>, pcr4),
        PE5: (pe5, 5, Input<Floating>, pcr5),
        PE6: (pe6, 6, Input<Floating>, pcr6),
        PE7: (pe7, 7, Input<Floating>, pcr7),
        PE8: (pe8, 8, Input<Floating>, pcr8),
        PE9: (pe9, 9, Input<Floating>, pcr9),
        PE10: (pe10, 10, Input<Floating>, pcr10),
        PE11: (pe11, 11, Input<Floating>, pcr11),
        PE12: (pe12, 12, Input<Floating>, pcr12),
        PE13: (pe13, 13, Input<Floating>, pcr13),
        PE14: (pe14, 14, Input<Floating>, pcr14),
        PE15: (pe15, 15, Input<Floating>, pcr15),
        PE16: (pe16, 16, Input<Floating>, pcr16),
        PE17: (pe17, 17, Input<Floating>, pcr17),
        PE18: (pe18, 18, Input<Floating>, pcr18),
        PE19: (pe19, 19, Input<Floating>, pcr19),
        PE20: (pe20, 20, Input<Floating>, pcr20),
        PE21: (pe21, 21, Input<Floating>, pcr21),
        PE22: (pe22, 22, Input<Floating>, pcr22),
        PE23: (pe23, 23, Input<Floating>, pcr23),
        PE24: (pe24, 24, Input<Floating>, pcr24),
        PE25: (pe25, 25, Input<Floating>, pcr25),
        PE26: (pe26, 26, Input<Floating>, pcr26),
        PE27: (pe27, 27, Input<Floating>, pcr27),
        PE28: (pe28, 28, Input<Floating>, pcr28),
        PE29: (pe29, 29, Input<Floating>, pcr29),
        PE30: (pe30, 30, Input<Floating>, pcr30),
        PE31: (pe31, 31, Input<Floating>, pcr31),
    ]
]);