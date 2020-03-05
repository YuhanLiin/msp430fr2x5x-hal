//! Prelude

pub use crate::capture::CaptureExt as _msp430fr2x5x_hal_CaptureExt;
pub use crate::capture::CapturePin as _msp430fr2x5x_hal_CapturePin;
pub use crate::clock::Clock as _msp430fr2x5x_hal_Clock;
pub use crate::clock::SmclkState as _msp430fr2x5x_hal_SmclkState;
pub use crate::gpio::Alternate1 as _msp430fr2x5x_hal_Alternate1;
pub use crate::gpio::Alternate2 as _msp430fr2x5x_hal_Alternate2;
pub use crate::gpio::Alternate3 as _msp430fr2x5x_hal_Alternate3;
pub use crate::gpio::GpioFunction as _msp430fr2x5x_hal_GpioFunction;
pub use crate::gpio::PinNum as _msp430fr2x5x_hal_PinNum;
pub use crate::pwm::PwmConfigChannels as _msp430fr2x5x_hal_PwmConfigChannels;
pub use crate::pwm::PwmExt as _msp430fr2x5x_hal_PwmExt;
pub use crate::pwm::PwmGpio as _msp430fr2x5x_hal_PwmGpio;
pub use crate::rtc::RtcClockSrc as _msp430fr2x5x_hal_RtcClockSrc;
pub use crate::serial::SerialUsci as _msp430fr2x5x_hal_SerialUsci;
pub use crate::timer::CapCmpPeriph as _msp430fr2x5x_hal_CapCmpPeriph;
pub use crate::timer::SevenCCRnTimer as _msp430fr2x5x_hal_SevenCCRnTimer;
pub use crate::timer::ThreeCCRnTimer as _msp430fr2x5x_hal_ThreeCCRnTimer;
pub use crate::timer::TimerPeriph as _msp430fr2x5x_hal_TimerPeriph;
pub use crate::watchdog::WatchdogSelect as _msp430fr2x5x_hal_WatchdogSelect;

pub use embedded_hal::prelude;
