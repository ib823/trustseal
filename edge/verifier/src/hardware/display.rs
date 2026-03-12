//! Display control for verifier UI.
//!
//! Supports small OLED/LCD displays via I2C or SPI.

use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Display errors.
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Display not initialized")]
    NotInitialized,

    #[error("I2C error: {0}")]
    I2cError(String),

    #[error("SPI error: {0}")]
    SpiError(String),

    #[error("Display error: {0}")]
    DeviceError(String),
}

/// Display interface type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayInterface {
    I2c { address: u8 },
    Spi { cs_pin: u8 },
}

/// Display driver type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayDriver {
    /// SSD1306 OLED (128x64 or 128x32).
    Ssd1306,
    /// ST7789 LCD (240x240 or 240x320).
    St7789,
    /// HD44780 character LCD.
    Hd44780,
}

/// Screen content.
#[derive(Debug, Clone)]
pub struct ScreenContent {
    /// Title line.
    pub title: Option<String>,
    /// Main message.
    pub message: String,
    /// Secondary message.
    pub secondary: Option<String>,
    /// Icon to display.
    pub icon: Option<DisplayIcon>,
}

/// Display icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIcon {
    Check,
    Cross,
    Warning,
    Clock,
    Wifi,
    WifiOff,
    Nfc,
    Ble,
}

/// Display controller.
pub struct Display {
    /// Display interface.
    interface: DisplayInterface,

    /// Display driver.
    driver: DisplayDriver,

    /// Display width.
    width: u16,

    /// Display height.
    height: u16,

    /// Current content.
    content: Arc<RwLock<Option<ScreenContent>>>,

    /// Whether display is initialized.
    initialized: Arc<RwLock<bool>>,
}

impl Display {
    /// Create a new display controller.
    pub fn new(interface: DisplayInterface, driver: DisplayDriver) -> Self {
        let (width, height) = match driver {
            DisplayDriver::Ssd1306 => (128, 64),
            DisplayDriver::St7789 => (240, 240),
            DisplayDriver::Hd44780 => (16, 2), // Characters, not pixels
        };

        Self {
            interface,
            driver,
            width,
            height,
            content: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the display.
    pub async fn init(&self) -> Result<(), DisplayError> {
        info!(
            "Initializing {:?} display via {:?}",
            self.driver, self.interface
        );

        #[cfg(feature = "rpi")]
        {
            // In production with embedded-graphics:
            // match self.interface {
            //     DisplayInterface::I2c { address } => {
            //         let i2c = rppal::i2c::I2c::new()
            //             .map_err(|e| DisplayError::I2cError(e.to_string()))?;
            //         // Initialize display driver...
            //     }
            //     DisplayInterface::Spi { cs_pin } => {
            //         let spi = rppal::spi::Spi::new(...)
            //             .map_err(|e| DisplayError::SpiError(e.to_string()))?;
            //         // Initialize display driver...
            //     }
            // }
        }

        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        self.clear().await?;

        Ok(())
    }

    /// Clear the display.
    pub async fn clear(&self) -> Result<(), DisplayError> {
        if !*self.initialized.read().await {
            return Err(DisplayError::NotInitialized);
        }

        debug!("Clearing display");

        let mut content = self.content.write().await;
        *content = None;

        // In production, would send clear command to display

        Ok(())
    }

    /// Show content on display.
    pub async fn show(&self, content: ScreenContent) -> Result<(), DisplayError> {
        if !*self.initialized.read().await {
            return Err(DisplayError::NotInitialized);
        }

        debug!("Displaying: {}", content.message);

        // In production, would render to display using embedded-graphics

        let mut current = self.content.write().await;
        *current = Some(content);

        Ok(())
    }

    /// Show "Ready" screen.
    pub async fn show_ready(&self, site_id: &str) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: Some("VaultPass".to_string()),
            message: "Ready".to_string(),
            secondary: Some(site_id.to_string()),
            icon: Some(DisplayIcon::Nfc),
        })
        .await
    }

    /// Show "Processing" screen.
    pub async fn show_processing(&self) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: None,
            message: "Verifying...".to_string(),
            secondary: None,
            icon: Some(DisplayIcon::Clock),
        })
        .await
    }

    /// Show "Granted" screen.
    pub async fn show_granted(&self, name: Option<&str>) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: None,
            message: "Access Granted".to_string(),
            secondary: name.map(String::from),
            icon: Some(DisplayIcon::Check),
        })
        .await
    }

    /// Show "Denied" screen.
    pub async fn show_denied(&self, reason: Option<&str>) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: None,
            message: "Access Denied".to_string(),
            secondary: reason.map(String::from),
            icon: Some(DisplayIcon::Cross),
        })
        .await
    }

    /// Show "Error" screen.
    pub async fn show_error(&self, message: &str) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: Some("Error".to_string()),
            message: message.to_string(),
            secondary: None,
            icon: Some(DisplayIcon::Warning),
        })
        .await
    }

    /// Show "Offline" indicator.
    pub async fn show_offline(&self) -> Result<(), DisplayError> {
        self.show(ScreenContent {
            title: Some("Offline Mode".to_string()),
            message: "Limited access".to_string(),
            secondary: None,
            icon: Some(DisplayIcon::WifiOff),
        })
        .await
    }

    /// Get display dimensions.
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Get display driver.
    pub fn driver(&self) -> DisplayDriver {
        self.driver
    }

    /// Check if initialized.
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    /// Get current content.
    pub async fn current_content(&self) -> Option<ScreenContent> {
        self.content.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_creation() {
        let display = Display::new(
            DisplayInterface::I2c { address: 0x3C },
            DisplayDriver::Ssd1306,
        );

        assert_eq!(display.dimensions(), (128, 64));
        assert_eq!(display.driver(), DisplayDriver::Ssd1306);
        assert!(!display.is_initialized().await);
    }

    #[tokio::test]
    async fn test_display_init() {
        let display = Display::new(
            DisplayInterface::I2c { address: 0x3C },
            DisplayDriver::Ssd1306,
        );

        display.init().await.unwrap();
        assert!(display.is_initialized().await);
    }

    #[tokio::test]
    async fn test_show_content() {
        let display = Display::new(
            DisplayInterface::I2c { address: 0x3C },
            DisplayDriver::Ssd1306,
        );

        display.init().await.unwrap();
        display.show_ready("VRF_01HXK").await.unwrap();

        let content = display.current_content().await.unwrap();
        assert_eq!(content.message, "Ready");
    }

    #[tokio::test]
    async fn test_not_initialized() {
        let display = Display::new(
            DisplayInterface::I2c { address: 0x3C },
            DisplayDriver::Ssd1306,
        );

        let result = display.show_ready("VRF_01HXK").await;
        assert!(matches!(result, Err(DisplayError::NotInitialized)));
    }

    #[tokio::test]
    async fn test_all_screens() {
        let display = Display::new(
            DisplayInterface::I2c { address: 0x3C },
            DisplayDriver::Ssd1306,
        );

        display.init().await.unwrap();

        display.show_ready("VRF_01HXK").await.unwrap();
        display.show_processing().await.unwrap();
        display.show_granted(Some("John Doe")).await.unwrap();
        display.show_denied(Some("Expired")).await.unwrap();
        display.show_error("Connection failed").await.unwrap();
        display.show_offline().await.unwrap();
        display.clear().await.unwrap();
    }
}
