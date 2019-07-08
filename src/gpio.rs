use core::marker::PhantomData;

use embedded_hal::digital::OutputPin;
use k64::{gpioa, GPIOA};
use rcc::AHB;

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





macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $gpioy:ident, $iopxenr:ident, $iopxrst:ident, $PXx:ident, [
        $($PXi:ident: ($pxi:ident, $i:ident, $MODE:ty, $AFR:ident),)+
    ]) => {
        pub mod $gpiox {
            use core::marker::PhantomData;

            use hal::digital::OutputPin;
            use k64::{$gpioy, $GPIOX};
            use rcc::AHB;


        }
    };
}