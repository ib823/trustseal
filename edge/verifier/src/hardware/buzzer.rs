//! Buzzer control for audio feedback.

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;
use tracing::debug;

use super::gpio::{GpioController, GpioError, PinMode};

/// Buzzer sound patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuzzerPattern {
    /// Single short beep (success).
    ShortBeep,
    /// Double short beep (attention).
    DoubleBeep,
    /// Long beep (error).
    LongBeep,
    /// Triple short beep (denied).
    TripleBeep,
    /// Success melody (ascending).
    SuccessMelody,
    /// Error melody (descending).
    ErrorMelody,
}

/// Buzzer controller.
pub struct Buzzer {
    /// GPIO controller.
    gpio: Arc<GpioController>,

    /// Buzzer pin.
    pin: u8,

    /// Default frequency (Hz).
    default_frequency: f64,

    /// Volume (0.0 - 1.0).
    volume: f64,

    /// Whether buzzer is enabled.
    enabled: bool,
}

impl Buzzer {
    /// Create a new buzzer controller.
    pub async fn new(gpio: Arc<GpioController>, pin: u8) -> Result<Self, GpioError> {
        gpio.configure(pin, PinMode::Output).await?;

        Ok(Self {
            gpio,
            pin,
            default_frequency: 2400.0, // Standard buzzer frequency
            volume: 0.5,
            enabled: true,
        })
    }

    /// Set volume (0.0 - 1.0).
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Enable/disable buzzer.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Play a tone for a duration.
    pub async fn tone(&self, frequency: f64, duration: Duration) -> Result<(), GpioError> {
        if !self.enabled {
            return Ok(());
        }

        debug!(
            "Playing tone: {} Hz for {:?}",
            frequency, duration
        );

        self.gpio.set_pwm(self.pin, frequency, self.volume).await?;
        sleep(duration).await;
        self.gpio.stop_pwm(self.pin).await?;

        Ok(())
    }

    /// Play a pattern.
    pub async fn play(&self, pattern: BuzzerPattern) -> Result<(), GpioError> {
        if !self.enabled {
            return Ok(());
        }

        debug!("Playing buzzer pattern: {:?}", pattern);

        match pattern {
            BuzzerPattern::ShortBeep => {
                self.tone(self.default_frequency, Duration::from_millis(100))
                    .await?;
            }

            BuzzerPattern::DoubleBeep => {
                self.tone(self.default_frequency, Duration::from_millis(80))
                    .await?;
                sleep(Duration::from_millis(80)).await;
                self.tone(self.default_frequency, Duration::from_millis(80))
                    .await?;
            }

            BuzzerPattern::LongBeep => {
                self.tone(self.default_frequency, Duration::from_millis(500))
                    .await?;
            }

            BuzzerPattern::TripleBeep => {
                for _ in 0..3 {
                    self.tone(self.default_frequency, Duration::from_millis(80))
                        .await?;
                    sleep(Duration::from_millis(80)).await;
                }
            }

            BuzzerPattern::SuccessMelody => {
                // Ascending tones
                self.tone(1000.0, Duration::from_millis(100)).await?;
                sleep(Duration::from_millis(50)).await;
                self.tone(1500.0, Duration::from_millis(100)).await?;
                sleep(Duration::from_millis(50)).await;
                self.tone(2000.0, Duration::from_millis(150)).await?;
            }

            BuzzerPattern::ErrorMelody => {
                // Descending tones
                self.tone(2000.0, Duration::from_millis(100)).await?;
                sleep(Duration::from_millis(50)).await;
                self.tone(1500.0, Duration::from_millis(100)).await?;
                sleep(Duration::from_millis(50)).await;
                self.tone(1000.0, Duration::from_millis(200)).await?;
            }
        }

        Ok(())
    }

    /// Simple beep.
    pub async fn beep(&self) -> Result<(), GpioError> {
        self.play(BuzzerPattern::ShortBeep).await
    }

    /// Success sound.
    pub async fn success(&self) -> Result<(), GpioError> {
        self.play(BuzzerPattern::SuccessMelody).await
    }

    /// Error sound.
    pub async fn error(&self) -> Result<(), GpioError> {
        self.play(BuzzerPattern::ErrorMelody).await
    }

    /// Denied sound (triple beep).
    pub async fn denied(&self) -> Result<(), GpioError> {
        self.play(BuzzerPattern::TripleBeep).await
    }

    /// Get GPIO pin.
    pub fn pin(&self) -> u8 {
        self.pin
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buzzer_creation() {
        let gpio = Arc::new(GpioController::new());
        let buzzer = Buzzer::new(gpio, 18).await.unwrap();

        assert_eq!(buzzer.pin(), 18);
        assert!(buzzer.is_enabled());
    }

    #[tokio::test]
    async fn test_buzzer_disabled() {
        let gpio = Arc::new(GpioController::new());
        let mut buzzer = Buzzer::new(gpio, 18).await.unwrap();

        buzzer.set_enabled(false);
        assert!(!buzzer.is_enabled());

        // Should not error when disabled
        buzzer.beep().await.unwrap();
    }

    #[tokio::test]
    async fn test_volume() {
        let gpio = Arc::new(GpioController::new());
        let mut buzzer = Buzzer::new(gpio, 18).await.unwrap();

        buzzer.set_volume(0.8);
        buzzer.set_volume(1.5); // Should clamp to 1.0
        buzzer.set_volume(-0.5); // Should clamp to 0.0
    }

    #[tokio::test]
    async fn test_patterns() {
        let gpio = Arc::new(GpioController::new());
        let buzzer = Buzzer::new(gpio, 18).await.unwrap();

        // Test all patterns (quick in simulation)
        buzzer.beep().await.unwrap();
        buzzer.success().await.unwrap();
        buzzer.error().await.unwrap();
        buzzer.denied().await.unwrap();
    }
}
