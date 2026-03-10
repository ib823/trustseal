import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nfc_manager/nfc_manager.dart';

/// NFC presentation state.
enum NfcState {
  idle,
  waitingForTag,
  reading,
  writing,
  complete,
  error,
}

/// NFC service for credential presentation.
///
/// Android: Uses Host Card Emulation (HCE)
/// - NDEF External Type record: domain `sahi.my`, type `vp`
/// - Payload: CBOR-encoded SD-JWT
///
/// iOS: NFC & SE Platform APIs (requires Apple entitlement)
/// - Minimum iOS 18.1 required
/// - Fallback: Universal Link URL scheme in NDEF record
class NfcService {
  final _stateController = StreamController<NfcState>.broadcast();

  Stream<NfcState> get stateStream => _stateController.stream;

  NfcState _currentState = NfcState.idle;
  NfcState get currentState => _currentState;

  void _setState(NfcState state) {
    _currentState = state;
    _stateController.add(state);
  }

  /// Check if NFC is available.
  Future<bool> isAvailable() async {
    return await NfcManager.instance.isAvailable();
  }

  /// Start NFC session for presenting credential.
  ///
  /// On Android: Emulates an NFC tag that the verifier can read.
  /// On iOS: Requires user to tap verifier device.
  Future<void> startPresentation(Uint8List cborPayload) async {
    if (!await isAvailable()) {
      _setState(NfcState.error);
      return;
    }

    _setState(NfcState.waitingForTag);

    if (Platform.isAndroid) {
      await _startAndroidHce(cborPayload);
    } else if (Platform.isIOS) {
      await _startIosSession(cborPayload);
    }
  }

  /// Android HCE implementation.
  ///
  /// Note: Full HCE implementation requires a custom HostApduService
  /// in native Android code. This is a simplified version using NDEF.
  Future<void> _startAndroidHce(Uint8List cborPayload) async {
    // Create NDEF record with VaultPass external type
    final ndefRecord = NdefRecord.createExternal(
      'sahi.my', // domain
      'vp', // type
      cborPayload,
    );

    final ndefMessage = NdefMessage([ndefRecord]);

    await NfcManager.instance.startSession(
      onDiscovered: (NfcTag tag) async {
        try {
          _setState(NfcState.writing);

          final ndef = Ndef.from(tag);
          if (ndef == null) {
            _setState(NfcState.error);
            return;
          }

          if (!ndef.isWritable) {
            _setState(NfcState.error);
            return;
          }

          await ndef.write(ndefMessage);
          _setState(NfcState.complete);
        } catch (e) {
          _setState(NfcState.error);
        } finally {
          await NfcManager.instance.stopSession();
        }
      },
      onError: (error) async {
        _setState(NfcState.error);
      },
    );
  }

  /// iOS NFC session implementation.
  ///
  /// Note: Full iOS 18.1+ NFC credential presentation requires
  /// Apple NFC & SE Platform entitlement. This is a simplified
  /// version using standard NDEF.
  Future<void> _startIosSession(Uint8List cborPayload) async {
    // Create NDEF record
    final ndefRecord = NdefRecord.createExternal(
      'sahi.my',
      'vp',
      cborPayload,
    );

    final ndefMessage = NdefMessage([ndefRecord]);

    await NfcManager.instance.startSession(
      alertMessage: 'Hold your phone near the verifier',
      onDiscovered: (NfcTag tag) async {
        try {
          _setState(NfcState.writing);

          final ndef = Ndef.from(tag);
          if (ndef == null || !ndef.isWritable) {
            _setState(NfcState.error);
            await NfcManager.instance.stopSession(errorMessage: 'Incompatible tag');
            return;
          }

          await ndef.write(ndefMessage);
          _setState(NfcState.complete);
          await NfcManager.instance.stopSession(alertMessage: 'Success');
        } catch (e) {
          _setState(NfcState.error);
          await NfcManager.instance.stopSession(errorMessage: 'Failed');
        }
      },
      onError: (error) async {
        _setState(NfcState.error);
      },
    );
  }

  /// Stop any active NFC session.
  Future<void> stopSession() async {
    try {
      await NfcManager.instance.stopSession();
    } catch (_) {
      // Ignore errors when stopping
    }
    _setState(NfcState.idle);
  }

  /// Clean up resources.
  void dispose() {
    stopSession();
    _stateController.close();
  }
}

/// Provider for NFC service.
final nfcServiceProvider = Provider<NfcService>((ref) {
  final service = NfcService();
  ref.onDispose(() => service.dispose());
  return service;
});
