pub use msp430fr2355 as pac;
/*         GPIO          */
mod gpio {
    use crate::gpio::*;
    use crate::pac::{P1, P2, P3, P4, P5, P6};
    use crate::hw_traits::gpio::gpio_impl; 

    // Define alternate pin transitions

    // P1 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P1, PIN, DIR> {}
    // P1 alternate 2
    impl<DIR>  ToAlternate2 for Pin<P1, Pin0, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin1, DIR> {}
    impl<PULL> ToAlternate2 for Pin<P1, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin6, DIR> {}
    impl<DIR>  ToAlternate2 for Pin<P1, Pin7, DIR> {}
    // P1 alternate 3
    impl<PIN: PinNum, DIR> ToAlternate3 for Pin<P1, PIN, DIR> {}

    // P2 alternate 1
    impl<DIR>  ToAlternate1 for Pin<P2, Pin0, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin1, DIR> {}
    impl<PULL> ToAlternate1 for Pin<P2, Pin2, Input<PULL>> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin3, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin6, DIR> {}
    impl<DIR>  ToAlternate1 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 2
    impl ToAlternate2 for Pin<P2, Pin0, Output> {}
    impl ToAlternate2 for Pin<P2, Pin1, Output> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin6, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P2, Pin7, DIR> {}
    // P2 alternate 3
    impl<DIR> ToAlternate3 for Pin<P2, Pin4, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P2, Pin5, DIR> {}

    // P3 alternate 1
    impl<DIR> ToAlternate1 for Pin<P3, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P3, Pin4, DIR> {}
    // P3 alternate 3
    impl<DIR> ToAlternate3 for Pin<P3, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin3, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin5, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin6, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P3, Pin7, DIR> {}

    // P4 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P4, PIN, DIR> {}
    // P4 alternate 2
    impl<DIR> ToAlternate2 for Pin<P4, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin2, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P4, Pin3, DIR> {}

    // P5 alternate 1
    impl<DIR> ToAlternate1 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate1 for Pin<P5, Pin3, DIR> {}
    // P5 alternate 2
    impl<DIR> ToAlternate2 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate2 for Pin<P5, Pin1, DIR> {}
    // P5 alternate 3
    impl<DIR> ToAlternate3 for Pin<P5, Pin0, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin1, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin2, DIR> {}
    impl<DIR> ToAlternate3 for Pin<P5, Pin3, DIR> {}

    // P6 alternate 1
    impl<PIN: PinNum, DIR> ToAlternate1 for Pin<P6, PIN, DIR> {}

    // GPIO port impls, PAC register methods, and marking ports as interrupt-capable
    gpio_impl!(p1: P1 => p1in, p1out, p1dir, p1ren, p1selc, p1sel0, p1sel1, [p1ies, p1ie, p1ifg, p1iv]);
    gpio_impl!(p2: P2 => p2in, p2out, p2dir, p2ren, p2selc, p2sel0, p2sel1, [p2ies, p2ie, p2ifg, p2iv]);
    gpio_impl!(p3: P3 => p3in, p3out, p3dir, p3ren, p3selc, p3sel0, p3sel1, [p3ies, p3ie, p3ifg, p3iv]);
    gpio_impl!(p4: P4 => p4in, p4out, p4dir, p4ren, p4selc, p4sel0, p4sel1, [p4ies, p4ie, p4ifg, p4iv]);
    gpio_impl!(p5: P5 => p5in, p5out, p5dir, p5ren, p5selc, p5sel0, p5sel1);
    gpio_impl!(p6: P6 => p6in, p6out, p6dir, p6ren, p6selc, p6sel0, p6sel1);
}
