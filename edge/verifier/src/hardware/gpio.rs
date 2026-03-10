//! GPIO control abstraction.

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// GPIO errors.
#[derive(Debug, Error)]
pub enum GpioError {
    #[error("Pin {0} not available")]
    PinNotAvailable(u8),

    #[error("Pin {0} already in use")]
    PinInUse(u8),

    #[error("Invalid pin mode for operation")]
    InvalidMode,

    #[error("Hardware error: {0}")]
    HardwareError(String),

    #[error("GPIO not available (simulation mode)")]
    NotAvailable,
}

/// Pin mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
}

/// Pin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    High,
    Low,
}

impl From<bool> for PinState {
    fn from(value: bool) -> Self {
        if value {
            PinState::High
        } else {
            PinState::Low
        }
    }
}

impl From<PinState> for bool {
    fn from(state: PinState) -> Self {
        matches!(state, PinState::High)
    }
}

/// Configured pin info.
struct PinConfig {
    mode: PinMode,
    state: PinState,
}

/// GPIO controller.
pub struct GpioController {
    /// Configured pins.
    pins: Arc<RwLock<HashMap<u8, PinConfig>>>,

    /// Whether running on real hardware.
    #[allow(dead_code)]
    is_hardware: bool,
}

impl GpioController {
    /// Create a new GPIO controller.
    pub fn new() -> Self {
        let is_hardware = cfg!(feature = "rpi");

        if is_hardware {
            info!("GPIO controller initialized for Raspberry Pi");
        } else {
            info!("GPIO controller initialized in simulation mode");
        }

        Self {
            pins: Arc::new(RwLock::new(HashMap::new())),
            is_hardware,
        }
    }

    /// Configure a pin.
    pub async fn configure(&self, pin: u8, mode: PinMode) -> Result<(), GpioError> {
        let mut pins = self.pins.write().await;

        if pins.contains_key(&pin) {
            return Err(GpioError::PinInUse(pin));
        }

        debug!("Configuring GPIO pin {} as {:?}", pin, mode);

        #[cfg(feature = "rpi")]
        {
            // In production with rppal:
            // let gpio = rppal::gpio::Gpio::new()
            //     .map_err(|e| GpioError::HardwareError(e.to_string()))?;
            // let pin = gpio.get(pin)
            //     .map_err(|e| GpioError::HardwareError(e.to_string()))?;
            // match mode {
            //     PinMode::Input => pin.into_input(),
            //     PinMode::Output => pin.into_output(),
            //     PinMode::InputPullUp => pin.into_input_pullup(),
            //     PinMode::InputPullDown => pin.into_input_pulldown(),
            // };
        }

        let initial_state = match mode {
            PinMode::Input | PinMode::InputPullUp | PinMode::InputPullDown => PinState::Low,
            PinMode::Output => PinState::Low,
        };

        pins.insert(
            pin,
            PinConfig {
                mode,
                state: initial_state,
            },
        );

        Ok(())
    }

    /// Set output pin state.
    pub async fn write(&self, pin: u8, state: PinState) -> Result<(), GpioError> {
        let mut pins = self.pins.write().await;

        let config = pins.get_mut(&pin).ok_or(GpioError::PinNotAvailable(pin))?;

        if config.mode != PinMode::Output {
            return Err(GpioError::InvalidMode);
        }

        debug!("Setting GPIO pin {} to {:?}", pin, state);

        #[cfg(feature = "rpi")]
        {
            // In production with rppal:
            // match state {
            //     PinState::High => pin.set_high(),
            //     PinState::Low => pin.set_low(),
            // }
        }

        config.state = state;
        Ok(())
    }

    /// Read input pin state.
    pub async fn read(&self, pin: u8) -> Result<PinState, GpioError> {
        let pins = self.pins.read().await;

        let config = pins.get(&pin).ok_or(GpioError::PinNotAvailable(pin))?;

        match config.mode {
            PinMode::Input | PinMode::InputPullUp | PinMode::InputPullDown => {
                #[cfg(feature = "rpi")]
                {
                    // In production with rppal:
                    // if pin.is_high() { PinState::High } else { PinState::Low }
                }

                // Simulation: return stored state
                Ok(config.state)
            }
            PinMode::Output => Err(GpioError::InvalidMode),
        }
    }

    /// Toggle output pin.
    pub async fn toggle(&self, pin: u8) -> Result<PinState, GpioError> {
        let current = {
            let pins = self.pins.read().await;
            let config = pins.get(&pin).ok_or(GpioError::PinNotAvailable(pin))?;
            config.state
        };

        let new_state = match current {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
        };

        self.write(pin, new_state).await?;
        Ok(new_state)
    }

    /// Release a pin.
    pub async fn release(&self, pin: u8) -> Result<(), GpioError> {
        let mut pins = self.pins.write().await;

        if pins.remove(&pin).is_none() {
            return Err(GpioError::PinNotAvailable(pin));
        }

        debug!("Released GPIO pin {}", pin);
        Ok(())
    }

    /// Set PWM on a pin (for buzzer/LED dimming).
    pub async fn set_pwm(&self, pin: u8, frequency: f64, duty_cycle: f64) -> Result<(), GpioError> {
        let pins = self.pins.read().await;

        let config = pins.get(&pin).ok_or(GpioError::PinNotAvailable(pin))?;

        if config.mode != PinMode::Output {
            return Err(GpioError::InvalidMode);
        }

        debug!(
            "Setting PWM on pin {}: freq={:.1}Hz, duty={:.1}%",
            pin,
            frequency,
            duty_cycle * 100.0
        );

        #[cfg(feature = "rpi")]
        {
            // In production with rppal:
            // pin.set_pwm_frequency(frequency, duty_cycle)
            //     .map_err(|e| GpioError::HardwareError(e.to_string()))?;
        }

        Ok(())
    }

    /// Stop PWM on a pin.
    pub async fn stop_pwm(&self, pin: u8) -> Result<(), GpioError> {
        self.write(pin, PinState::Low).await
    }

    /// Get list of configured pins.
    pub async fn configured_pins(&self) -> Vec<u8> {
        self.pins.read().await.keys().copied().collect()
    }
}

impl Default for GpioController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_configure_pin() {
        let gpio = GpioController::new();

        gpio.configure(17, PinMode::Output).await.unwrap();

        let pins = gpio.configured_pins().await;
        assert!(pins.contains(&17));
    }

    #[tokio::test]
    async fn test_write_read() {
        let gpio = GpioController::new();

        gpio.configure(17, PinMode::Output).await.unwrap();
        gpio.write(17, PinState::High).await.unwrap();

        // Output pins can't be read in normal mode
        let result = gpio.read(17).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_input_pin() {
        let gpio = GpioController::new();

        gpio.configure(18, PinMode::Input).await.unwrap();

        let state = gpio.read(18).await.unwrap();
        assert_eq!(state, PinState::Low);
    }

    #[tokio::test]
    async fn test_toggle() {
        let gpio = GpioController::new();

        gpio.configure(17, PinMode::Output).await.unwrap();

        let state = gpio.toggle(17).await.unwrap();
        assert_eq!(state, PinState::High);

        let state = gpio.toggle(17).await.unwrap();
        assert_eq!(state, PinState::Low);
    }

    #[tokio::test]
    async fn test_pin_in_use() {
        let gpio = GpioController::new();

        gpio.configure(17, PinMode::Output).await.unwrap();

        let result = gpio.configure(17, PinMode::Input).await;
        assert!(matches!(result, Err(GpioError::PinInUse(17))));
    }

    #[tokio::test]
    async fn test_release_pin() {
        let gpio = GpioController::new();

        gpio.configure(17, PinMode::Output).await.unwrap();
        gpio.release(17).await.unwrap();

        // Should be able to reconfigure
        gpio.configure(17, PinMode::Input).await.unwrap();
    }
}
