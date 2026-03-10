//! Tamper detection for physical security.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use super::gpio::{GpioController, GpioError, PinMode, PinState};

/// Tamper event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TamperEvent {
    /// Case opened.
    CaseOpened,
    /// Case closed.
    CaseClosed,
    /// Device tilted/moved.
    MotionDetected,
    /// Multiple failed attempts.
    BruteForceAttempt { count: u32 },
    /// Unknown tamper.
    Unknown,
}

/// Tamper detector configuration.
#[derive(Debug, Clone)]
pub struct TamperConfig {
    /// Case switch pin.
    pub case_switch_pin: Option<u8>,
    /// Motion sensor pin.
    pub motion_pin: Option<u8>,
    /// Debounce duration.
    pub debounce_ms: u64,
    /// Brute force threshold.
    pub brute_force_threshold: u32,
    /// Brute force window.
    pub brute_force_window: Duration,
}

impl Default for TamperConfig {
    fn default() -> Self {
        Self {
            case_switch_pin: Some(4),
            motion_pin: None,
            debounce_ms: 50,
            brute_force_threshold: 5,
            brute_force_window: Duration::from_secs(60),
        }
    }
}

/// Tamper detector.
pub struct TamperDetector {
    /// GPIO controller.
    gpio: Arc<GpioController>,

    /// Configuration.
    config: TamperConfig,

    /// Event sender.
    event_tx: mpsc::Sender<TamperEvent>,

    /// Running state.
    running: Arc<RwLock<bool>>,

    /// Failed attempt tracking.
    failed_attempts: Arc<RwLock<Vec<Instant>>>,
}

impl TamperDetector {
    /// Create a new tamper detector.
    pub async fn new(
        gpio: Arc<GpioController>,
        config: TamperConfig,
    ) -> Result<(Self, mpsc::Receiver<TamperEvent>), GpioError> {
        let (event_tx, event_rx) = mpsc::channel(32);

        // Configure case switch pin
        if let Some(pin) = config.case_switch_pin {
            gpio.configure(pin, PinMode::InputPullUp).await?;
        }

        // Configure motion sensor pin
        if let Some(pin) = config.motion_pin {
            gpio.configure(pin, PinMode::Input).await?;
        }

        let detector = Self {
            gpio,
            config,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            failed_attempts: Arc::new(RwLock::new(Vec::new())),
        };

        Ok((detector, event_rx))
    }

    /// Start monitoring.
    pub async fn start(&self) -> Result<(), GpioError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;

        info!("Starting tamper detection");

        // Spawn monitoring task
        let gpio = self.gpio.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            let mut last_case_state: Option<PinState> = None;
            let debounce = Duration::from_millis(config.debounce_ms);

            while *running_flag.read().await {
                // Check case switch
                if let Some(pin) = config.case_switch_pin {
                    if let Ok(state) = gpio.read(pin).await {
                        if let Some(last) = last_case_state {
                            if state != last {
                                let event = if state == PinState::High {
                                    TamperEvent::CaseOpened
                                } else {
                                    TamperEvent::CaseClosed
                                };
                                warn!("Tamper detected: {:?}", event);
                                let _ = event_tx.send(event).await;
                            }
                        }
                        last_case_state = Some(state);
                    }
                }

                // Check motion sensor
                if let Some(pin) = config.motion_pin {
                    if let Ok(state) = gpio.read(pin).await {
                        if state == PinState::High {
                            debug!("Motion detected");
                            let _ = event_tx.send(TamperEvent::MotionDetected).await;
                        }
                    }
                }

                tokio::time::sleep(debounce).await;
            }
        });

        Ok(())
    }

    /// Stop monitoring.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopped tamper detection");
    }

    /// Record a failed authentication attempt.
    pub async fn record_failed_attempt(&self) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Instant::now();

        // Remove old attempts outside window
        attempts.retain(|t| now.duration_since(*t) < self.config.brute_force_window);

        // Add new attempt
        attempts.push(now);

        let count = attempts.len() as u32;
        debug!("Failed attempt recorded, count: {}", count);

        // Check threshold
        if count >= self.config.brute_force_threshold {
            warn!("Brute force attempt detected: {} attempts", count);
            let _ = self
                .event_tx
                .send(TamperEvent::BruteForceAttempt { count })
                .await;
        }
    }

    /// Get current failed attempt count.
    pub async fn failed_attempt_count(&self) -> u32 {
        let attempts = self.failed_attempts.read().await;
        let now = Instant::now();
        attempts
            .iter()
            .filter(|t| now.duration_since(**t) < self.config.brute_force_window)
            .count() as u32
    }

    /// Clear failed attempts.
    pub async fn clear_failed_attempts(&self) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.clear();
    }

    /// Check if case is open.
    pub async fn is_case_open(&self) -> Result<bool, GpioError> {
        if let Some(pin) = self.config.case_switch_pin {
            let state = self.gpio.read(pin).await?;
            Ok(state == PinState::High)
        } else {
            Ok(false)
        }
    }

    /// Check if monitoring is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tamper_detector_creation() {
        let gpio = Arc::new(GpioController::new());
        let config = TamperConfig::default();

        let (detector, _rx) = TamperDetector::new(gpio, config).await.unwrap();
        assert!(!detector.is_running().await);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let gpio = Arc::new(GpioController::new());
        let config = TamperConfig {
            case_switch_pin: None,
            motion_pin: None,
            ..Default::default()
        };

        let (detector, _rx) = TamperDetector::new(gpio, config).await.unwrap();

        detector.start().await.unwrap();
        assert!(detector.is_running().await);

        detector.stop().await;
        // Give task time to stop
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!detector.is_running().await);
    }

    #[tokio::test]
    async fn test_failed_attempts() {
        let gpio = Arc::new(GpioController::new());
        let config = TamperConfig {
            case_switch_pin: None,
            motion_pin: None,
            brute_force_threshold: 3,
            brute_force_window: Duration::from_secs(60),
            ..Default::default()
        };

        let (detector, mut rx) = TamperDetector::new(gpio, config).await.unwrap();

        detector.record_failed_attempt().await;
        assert_eq!(detector.failed_attempt_count().await, 1);

        detector.record_failed_attempt().await;
        assert_eq!(detector.failed_attempt_count().await, 2);

        detector.record_failed_attempt().await;
        assert_eq!(detector.failed_attempt_count().await, 3);

        // Should have triggered brute force event
        let event = rx.try_recv();
        assert!(matches!(
            event,
            Ok(TamperEvent::BruteForceAttempt { count: 3 })
        ));
    }

    #[tokio::test]
    async fn test_clear_attempts() {
        let gpio = Arc::new(GpioController::new());
        let config = TamperConfig {
            case_switch_pin: None,
            motion_pin: None,
            ..Default::default()
        };

        let (detector, _rx) = TamperDetector::new(gpio, config).await.unwrap();

        detector.record_failed_attempt().await;
        detector.record_failed_attempt().await;
        assert_eq!(detector.failed_attempt_count().await, 2);

        detector.clear_failed_attempts().await;
        assert_eq!(detector.failed_attempt_count().await, 0);
    }
}
