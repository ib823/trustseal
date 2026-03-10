//! VaultPass Edge Verifier
//!
//! Raspberry Pi-based credential verification device for physical access control.
//!
//! Features:
//! - BLE GATT server for wallet credential presentation
//! - NFC reader for contactless credential verification
//! - Offline operation with cached status lists
//! - Hardware GPIO control for door locks, LEDs, buzzer
//! - MQTT connectivity for real-time updates
//! - Append-only audit log for all access decisions

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![deny(unused_imports)]

mod audit;
mod ble;
mod config;
mod crypto;
mod hardware;
mod mqtt;
mod nfc;
mod policy;
mod revocation;

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::audit::AuditLog;
use crate::ble::{BleAdvertiser, GattServer, PresentationResponse};
use crate::config::VerifierConfig;
use crate::crypto::{DidResolver, SdJwtVerifier};
use crate::hardware::{Buzzer, GpioController, StatusLeds};
use crate::mqtt::{MqttClient, MqttConfig};
use crate::nfc::{NfcEvent, NfcReader};
use crate::policy::PolicyEngine;
use crate::revocation::{RevocationCache, RevocationSync, SyncConfig};

/// Shared application state.
pub struct AppState {
    pub config: VerifierConfig,
    pub policy: PolicyEngine,
    pub revocation: Arc<RevocationCache>,
    pub audit: AuditLog,
    pub sd_jwt_verifier: SdJwtVerifier,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("edge_verifier=info".parse()?))
        .init();

    info!("VaultPass Edge Verifier starting...");

    // Load configuration
    let config = VerifierConfig::load()?;
    info!(
        "Configuration loaded: site_id={}, property_id={}",
        config.site_id, config.property_id
    );

    // Initialize audit log
    let audit = AuditLog::new(&config.audit_db_path)?;
    info!("Audit log initialized");

    // Initialize revocation cache
    let revocation = Arc::new(RevocationCache::new(config.status_list_ttl));
    info!(
        "Revocation cache initialized (TTL: {}s)",
        config.status_list_ttl.as_secs()
    );

    // Initialize policy engine
    let policy = PolicyEngine::new(&config);
    info!("Policy engine initialized");

    // Initialize DID resolver and SD-JWT verifier
    let did_resolver = DidResolver::new().with_ttl(config.trust_registry_ttl);
    let sd_jwt_verifier = SdJwtVerifier::new(did_resolver);
    info!("Crypto verifier initialized");

    // Initialize GPIO and hardware
    let gpio = Arc::new(GpioController::new());
    let leds = StatusLeds::new(
        gpio.clone(),
        config.hardware.red_led_pin,
        config.hardware.green_led_pin,
        Some(config.hardware.amber_led_pin),
    )
    .await?;
    let buzzer = Buzzer::new(gpio.clone(), config.hardware.buzzer_pin).await?;
    info!("Hardware initialized");

    // Create shared state
    let state = Arc::new(RwLock::new(AppState {
        config: config.clone(),
        policy,
        revocation: revocation.clone(),
        audit,
        sd_jwt_verifier,
    }));

    // Start revocation sync
    let sync_config = SyncConfig {
        poll_interval: Duration::from_secs(300),
        api_base_url: config.api_base_url.clone(),
        enabled: true,
    };
    let revocation_sync = Arc::new(RevocationSync::new(revocation.clone(), sync_config));
    let sync_handle = {
        let sync = revocation_sync.clone();
        tokio::spawn(async move {
            sync.run_polling().await;
        })
    };

    // Start MQTT client
    let mqtt_config = MqttConfig {
        broker_url: config.mqtt_broker_url.clone(),
        client_id: config.mqtt_client_id.clone().unwrap_or_else(|| config.site_id.clone()),
        ..Default::default()
    };
    let (mqtt_client, mut mqtt_rx) = MqttClient::new(mqtt_config);
    let mqtt_client = Arc::new(mqtt_client);

    let mqtt_handle = {
        let client = mqtt_client.clone();
        let site_id = config.site_id.clone();
        let property_id = config.property_id.clone();
        tokio::spawn(async move {
            if let Err(e) = client.connect().await {
                error!("MQTT connection failed: {}", e);
                return;
            }
            if let Err(e) = client
                .subscribe_verifier_topics(&site_id, &property_id)
                .await
            {
                error!("MQTT subscription failed: {}", e);
            }
            info!("MQTT client connected");

            // Handle incoming messages
            while let Some(msg) = mqtt_rx.recv().await {
                info!("MQTT message on {}: {} bytes", msg.topic, msg.payload.len());
                // Process status list updates, policy updates, etc.
            }
        })
    };

    // Start BLE advertiser and GATT server
    let advertiser = BleAdvertiser::new(&config.site_id, &config.property_id);
    let (gatt_server, mut gatt_rx) = GattServer::new(&config.site_id);

    let ble_handle = {
        let state = state.clone();
        let advertiser = Arc::new(advertiser);
        let gatt = Arc::new(gatt_server);

        tokio::spawn(async move {
            if let Err(e) = advertiser.start().await {
                error!("BLE advertising failed: {}", e);
                return;
            }
            if let Err(e) = gatt.start().await {
                error!("GATT server failed: {}", e);
                return;
            }
            info!("BLE server started");

            // Handle presentation events
            while let Some(event) = gatt_rx.recv().await {
                let app_state = state.read().await;
                let response = process_presentation(&app_state, &event.request.sd_jwt).await;
                let _ = event.response_tx.send(response).await;
            }
        })
    };

    // Start NFC reader
    let (nfc_reader, mut nfc_rx) = NfcReader::new();
    let nfc_handle = {
        let state = state.clone();
        let reader = Arc::new(nfc_reader);

        tokio::spawn(async move {
            if let Err(e) = reader.start().await {
                error!("NFC reader failed: {}", e);
                return;
            }
            info!("NFC reader started");

            // Handle NFC events
            while let Some(event) = nfc_rx.recv().await {
                match event {
                    NfcEvent::CredentialDetected { sd_jwt } => {
                        let app_state = state.read().await;
                        let _response = process_presentation(&app_state, &sd_jwt).await;
                    }
                    _ => {}
                }
            }
        })
    };

    // Set up signal handling for graceful shutdown
    let shutdown_handle = tokio::spawn(async move {
        wait_for_shutdown().await;
        info!("Shutdown signal received, cleaning up...");
    });

    // Signal ready
    leds.show_ready().await?;
    buzzer.beep().await?;

    // Log startup complete
    {
        let mut app = state.write().await;
        app.audit.log_startup(&config.site_id).await?;
    }

    info!("Edge verifier ready and listening");

    // Wait for any task to complete (shutdown or error)
    tokio::select! {
        _ = shutdown_handle => {
            info!("Shutdown complete");
        }
        result = mqtt_handle => {
            if let Err(e) = result {
                error!("MQTT task panicked: {}", e);
            }
        }
        result = ble_handle => {
            if let Err(e) = result {
                error!("BLE task panicked: {}", e);
            }
        }
        result = nfc_handle => {
            if let Err(e) = result {
                error!("NFC task panicked: {}", e);
            }
        }
        _ = sync_handle => {
            warn!("Sync task ended");
        }
    }

    // Cleanup
    leds.all_off().await?;

    Ok(())
}

/// Process a credential presentation.
async fn process_presentation(state: &AppState, sd_jwt: &str) -> PresentationResponse {
    // Verify SD-JWT
    let verification = match state
        .sd_jwt_verifier
        .verify(sd_jwt, None, Some(&state.config.site_id))
        .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!("SD-JWT verification failed: {}", e);
            return PresentationResponse::invalid(&e.to_string(), "");
        }
    };

    // Check revocation status
    if let Some(status) = &verification.status {
        match state
            .revocation
            .is_revoked(&status.status_list_credential, status.status_list_index)
            .await
        {
            Ok(true) => {
                warn!("Credential revoked");
                return PresentationResponse::revoked("");
            }
            Ok(false) => {}
            Err(e) => {
                error!("Revocation check failed: {}", e);
                // Fail closed - deny on error
                return PresentationResponse::denied("Revocation check failed", "");
            }
        }
    }

    // Check expiration
    if let Some(exp) = verification.exp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;
        if now > exp {
            return PresentationResponse::expired("");
        }
    }

    // Evaluate policy
    let decision = state
        .policy
        .evaluate(
            verification.holder_did.as_deref(),
            Some(&verification.issuer_did),
            verification.credential_type.as_deref(),
            None,
            None,
        )
        .await;

    match decision {
        policy::AccessDecision::Allow | policy::AccessDecision::AllowWithLog => {
            PresentationResponse::granted("")
        }
        policy::AccessDecision::Deny => PresentationResponse::denied("Access denied by policy", ""),
    }
}

/// Wait for shutdown signal (SIGINT or SIGTERM).
async fn wait_for_shutdown() {
    use futures::StreamExt;
    use signal_hook::consts::{SIGINT, SIGTERM};
    use signal_hook_tokio::Signals;

    let mut signals = Signals::new([SIGINT, SIGTERM]).expect("Failed to register signals");

    if let Some(signal) = signals.next().await {
        match signal {
            SIGINT => info!("Received SIGINT"),
            SIGTERM => info!("Received SIGTERM"),
            _ => {}
        }
    }
}
