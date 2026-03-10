//! Hardware control for Raspberry Pi verifier.
//!
//! Controls GPIO for LEDs, buzzer, display, and tamper detection.

mod buzzer;
mod display;
mod gpio;
mod led;
mod tamper;

pub use buzzer::Buzzer;
pub use gpio::GpioController;
pub use led::StatusLeds;
