//! LED control for status indication.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::debug;

use super::gpio::{GpioController, GpioError, PinMode, PinState};

/// LED color (for multi-color LED or separate LEDs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedColor {
    Red,
    Green,
    Blue,
    Yellow,
    White,
}

/// LED pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedPattern {
    /// Solid on.
    Solid,
    /// Single blink.
    Blink,
    /// Fast blinking.
    FastBlink,
    /// Slow blinking.
    SlowBlink,
    /// Pulse (fade in/out).
    Pulse,
    /// Off.
    Off,
}

/// LED controller.
pub struct Led {
    /// GPIO controller.
    gpio: Arc<GpioController>,

    /// Pin for this LED.
    pin: u8,

    /// Current color.
    color: LedColor,

    /// Current pattern.
    pattern: Arc<RwLock<LedPattern>>,

    /// Whether pattern loop is running.
    running: Arc<RwLock<bool>>,
}

impl Led {
    /// Create a new LED controller.
    pub async fn new(gpio: Arc<GpioController>, pin: u8, color: LedColor) -> Result<Self, GpioError> {
        gpio.configure(pin, PinMode::Output).await?;

        Ok(Self {
            gpio,
            pin,
            color,
            pattern: Arc::new(RwLock::new(LedPattern::Off)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Set LED pattern.
    pub async fn set_pattern(&self, pattern: LedPattern) -> Result<(), GpioError> {
        let mut current = self.pattern.write().await;
        *current = pattern;

        debug!("LED {} ({:?}) set to {:?}", self.pin, self.color, pattern);

        match pattern {
            LedPattern::Solid => {
                self.gpio.write(self.pin, PinState::High).await?;
            }
            LedPattern::Off => {
                self.gpio.write(self.pin, PinState::Low).await?;
            }
            LedPattern::Blink => {
                // Single blink
                self.gpio.write(self.pin, PinState::High).await?;
                sleep(Duration::from_millis(100)).await;
                self.gpio.write(self.pin, PinState::Low).await?;
            }
            _ => {
                // Pattern handled by run_pattern
            }
        }

        Ok(())
    }

    /// Run continuous pattern (spawns task).
    pub fn start_pattern(&self, pattern: LedPattern) {
        let gpio = self.gpio.clone();
        let pin = self.pin;
        let pattern_state = self.pattern.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            {
                let mut r = running.write().await;
                *r = true;
            }
            {
                let mut p = pattern_state.write().await;
                *p = pattern;
            }

            loop {
                let current_pattern = *pattern_state.read().await;
                let is_running = *running.read().await;

                if !is_running || current_pattern == LedPattern::Off {
                    let _ = gpio.write(pin, PinState::Low).await;
                    break;
                }

                match current_pattern {
                    LedPattern::FastBlink => {
                        let _ = gpio.toggle(pin).await;
                        sleep(Duration::from_millis(100)).await;
                    }
                    LedPattern::SlowBlink => {
                        let _ = gpio.toggle(pin).await;
                        sleep(Duration::from_millis(500)).await;
                    }
                    LedPattern::Pulse => {
                        // Simulate pulse with PWM
                        for duty in (0..=100).step_by(5) {
                            let _ = gpio.set_pwm(pin, 1000.0, duty as f64 / 100.0).await;
                            sleep(Duration::from_millis(20)).await;
                        }
                        for duty in (0..=100).rev().step_by(5) {
                            let _ = gpio.set_pwm(pin, 1000.0, duty as f64 / 100.0).await;
                            sleep(Duration::from_millis(20)).await;
                        }
                    }
                    LedPattern::Solid => {
                        let _ = gpio.write(pin, PinState::High).await;
                        sleep(Duration::from_millis(100)).await;
                    }
                    _ => break,
                }
            }

            let mut r = running.write().await;
            *r = false;
        });
    }

    /// Stop pattern.
    pub async fn stop(&self) -> Result<(), GpioError> {
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        {
            let mut pattern = self.pattern.write().await;
            *pattern = LedPattern::Off;
        }
        self.gpio.write(self.pin, PinState::Low).await
    }

    /// Turn on solid.
    pub async fn on(&self) -> Result<(), GpioError> {
        self.set_pattern(LedPattern::Solid).await
    }

    /// Turn off.
    pub async fn off(&self) -> Result<(), GpioError> {
        self.set_pattern(LedPattern::Off).await
    }

    /// Get current pattern.
    pub async fn current_pattern(&self) -> LedPattern {
        *self.pattern.read().await
    }

    /// Get LED color.
    pub fn color(&self) -> LedColor {
        self.color
    }

    /// Get GPIO pin.
    pub fn pin(&self) -> u8 {
        self.pin
    }
}

/// Status LED set (red, green, optionally blue).
pub struct StatusLeds {
    /// Red LED (error/denied).
    pub red: Led,
    /// Green LED (success/granted).
    pub green: Led,
    /// Blue LED (processing/ready).
    pub blue: Option<Led>,
}

impl StatusLeds {
    /// Create status LEDs.
    pub async fn new(
        gpio: Arc<GpioController>,
        red_pin: u8,
        green_pin: u8,
        blue_pin: Option<u8>,
    ) -> Result<Self, GpioError> {
        let red = Led::new(gpio.clone(), red_pin, LedColor::Red).await?;
        let green = Led::new(gpio.clone(), green_pin, LedColor::Green).await?;
        let blue = if let Some(pin) = blue_pin {
            Some(Led::new(gpio, pin, LedColor::Blue).await?)
        } else {
            None
        };

        Ok(Self { red, green, blue })
    }

    /// Show "ready" state (blue solid or green slow blink).
    pub async fn show_ready(&self) -> Result<(), GpioError> {
        self.red.off().await?;
        self.green.off().await?;

        if let Some(blue) = &self.blue {
            blue.on().await?;
        } else {
            self.green.start_pattern(LedPattern::SlowBlink);
        }

        Ok(())
    }

    /// Show "processing" state (blue fast blink).
    pub async fn show_processing(&self) -> Result<(), GpioError> {
        self.red.off().await?;
        self.green.off().await?;

        if let Some(blue) = &self.blue {
            blue.start_pattern(LedPattern::FastBlink);
        } else {
            self.green.start_pattern(LedPattern::FastBlink);
        }

        Ok(())
    }

    /// Show "granted" state (green solid).
    pub async fn show_granted(&self) -> Result<(), GpioError> {
        self.red.off().await?;
        if let Some(blue) = &self.blue {
            blue.off().await?;
        }
        self.green.on().await
    }

    /// Show "denied" state (red solid).
    pub async fn show_denied(&self) -> Result<(), GpioError> {
        self.green.off().await?;
        if let Some(blue) = &self.blue {
            blue.off().await?;
        }
        self.red.on().await
    }

    /// Show "error" state (red fast blink).
    pub async fn show_error(&self) -> Result<(), GpioError> {
        self.green.off().await?;
        if let Some(blue) = &self.blue {
            blue.off().await?;
        }
        self.red.start_pattern(LedPattern::FastBlink);
        Ok(())
    }

    /// Turn all off.
    pub async fn all_off(&self) -> Result<(), GpioError> {
        self.red.off().await?;
        self.green.off().await?;
        if let Some(blue) = &self.blue {
            blue.off().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_led_creation() {
        let gpio = Arc::new(GpioController::new());
        let led = Led::new(gpio, 17, LedColor::Red).await.unwrap();

        assert_eq!(led.color(), LedColor::Red);
        assert_eq!(led.pin(), 17);
        assert_eq!(led.current_pattern().await, LedPattern::Off);
    }

    #[tokio::test]
    async fn test_led_on_off() {
        let gpio = Arc::new(GpioController::new());
        let led = Led::new(gpio, 17, LedColor::Green).await.unwrap();

        led.on().await.unwrap();
        assert_eq!(led.current_pattern().await, LedPattern::Solid);

        led.off().await.unwrap();
        assert_eq!(led.current_pattern().await, LedPattern::Off);
    }

    #[tokio::test]
    async fn test_status_leds() {
        let gpio = Arc::new(GpioController::new());
        let leds = StatusLeds::new(gpio, 17, 27, Some(22)).await.unwrap();

        leds.show_ready().await.unwrap();
        leds.show_processing().await.unwrap();
        leds.show_granted().await.unwrap();
        leds.show_denied().await.unwrap();
        leds.all_off().await.unwrap();
    }
}
