import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_blue_plus/flutter_blue_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// BLE service UUIDs for VaultPass.
///
/// GATT service design per spec:
/// - Service UUID: Custom 128-bit
/// - Challenge characteristic (Read): Verifier publishes fresh nonce
/// - Presentation characteristic (Write): Wallet writes SD-JWT VP
/// - Result characteristic (Notify): Verifier sends GRANTED/DENIED
abstract class BleUuids {
  /// VaultPass verifier service UUID.
  static final serviceUuid = Guid('SAHI0001-0000-1000-8000-00805F9B34FB');

  /// Challenge characteristic (Read).
  static final challengeUuid = Guid('SAHI0002-0000-1000-8000-00805F9B34FB');

  /// Presentation characteristic (Write).
  static final presentationUuid = Guid('SAHI0003-0000-1000-8000-00805F9B34FB');

  /// Result characteristic (Notify).
  static final resultUuid = Guid('SAHI0004-0000-1000-8000-00805F9B34FB');
}

/// Presentation result from verifier.
enum PresentationResult {
  granted,
  denied,
  error,
  timeout,
}

/// BLE presentation state.
enum BleState {
  idle,
  scanning,
  connecting,
  discovering,
  readingChallenge,
  presenting,
  awaitingResult,
  complete,
  error,
}

/// BLE service for credential presentation.
///
/// Optimizations per spec (<200ms target):
/// 1. Request ConnectionPriority.high (7.5ms interval)
/// 2. Negotiate MTU 512 (fit presentation in 1-2 packets)
/// 3. Bond devices for fast reconnection (<50ms)
/// 4. Pre-filter scans by service UUID
/// 5. ScanMode.lowLatency on detection
class BleService {
  final _stateController = StreamController<BleState>.broadcast();
  final _resultController = StreamController<PresentationResult>.broadcast();

  BluetoothDevice? _connectedDevice;
  StreamSubscription<List<ScanResult>>? _scanSubscription;
  StreamSubscription<BluetoothConnectionState>? _connectionSubscription;
  StreamSubscription<List<int>>? _resultSubscription;

  Stream<BleState> get stateStream => _stateController.stream;
  Stream<PresentationResult> get resultStream => _resultController.stream;

  BleState _currentState = BleState.idle;
  BleState get currentState => _currentState;

  void _setState(BleState state) {
    _currentState = state;
    _stateController.add(state);
  }

  /// Check if Bluetooth is available and on.
  Future<bool> isAvailable() async {
    if (!await FlutterBluePlus.isSupported) {
      return false;
    }
    final state = await FlutterBluePlus.adapterState.first;
    return state == BluetoothAdapterState.on;
  }

  /// Start scanning for VaultPass verifiers.
  Future<void> startScanning() async {
    if (_currentState == BleState.scanning) return;

    _setState(BleState.scanning);

    // Pre-filter scans by service UUID for efficiency
    await FlutterBluePlus.startScan(
      withServices: [BleUuids.serviceUuid],
      timeout: const Duration(seconds: 10),
      androidScanMode: AndroidScanMode.lowLatency, // Fastest scan mode
    );

    _scanSubscription = FlutterBluePlus.scanResults.listen((results) {
      if (results.isNotEmpty) {
        // Found a verifier - connect immediately
        stopScanning();
        _connectToDevice(results.first.device);
      }
    });
  }

  /// Stop scanning.
  Future<void> stopScanning() async {
    await FlutterBluePlus.stopScan();
    await _scanSubscription?.cancel();
    _scanSubscription = null;
    if (_currentState == BleState.scanning) {
      _setState(BleState.idle);
    }
  }

  /// Connect to a verifier device.
  Future<void> _connectToDevice(BluetoothDevice device) async {
    _setState(BleState.connecting);
    _connectedDevice = device;

    try {
      // Monitor connection state
      _connectionSubscription = device.connectionState.listen((state) {
        if (state == BluetoothConnectionState.disconnected) {
          _handleDisconnection();
        }
      });

      // Connect with auto-connect for bonded devices
      await device.connect(
        autoConnect: false,
        timeout: const Duration(seconds: 5),
      );

      // Request high connection priority for <200ms presentation
      await device.requestConnectionPriority(
        connectionPriorityRequest: ConnectionPriority.high,
      );

      // Request MTU 512 to fit presentation in 1-2 packets
      await device.requestMtu(512);
    } catch (e) {
      _setState(BleState.error);
      _resultController.add(PresentationResult.error);
      return;
    }
  }

  /// Present a credential to the connected verifier.
  ///
  /// Flow:
  /// 1. Discover services
  /// 2. Read challenge (nonce)
  /// 3. Sign VP with nonce (done by caller)
  /// 4. Write presentation
  /// 5. Subscribe to result
  /// 6. Return result
  Future<PresentationResult> present(Uint8List signedPresentation) async {
    if (_connectedDevice == null) {
      return PresentationResult.error;
    }

    final device = _connectedDevice!;
    final completer = Completer<PresentationResult>();

    try {
      // Discover services
      _setState(BleState.discovering);
      final services = await device.discoverServices();

      final service = services.firstWhere(
        (s) => s.uuid == BleUuids.serviceUuid,
        orElse: () => throw Exception('VaultPass service not found'),
      );

      final challengeChar = service.characteristics.firstWhere(
        (c) => c.uuid == BleUuids.challengeUuid,
        orElse: () => throw Exception('Challenge characteristic not found'),
      );

      final presentationChar = service.characteristics.firstWhere(
        (c) => c.uuid == BleUuids.presentationUuid,
        orElse: () => throw Exception('Presentation characteristic not found'),
      );

      final resultChar = service.characteristics.firstWhere(
        (c) => c.uuid == BleUuids.resultUuid,
        orElse: () => throw Exception('Result characteristic not found'),
      );

      // Read challenge
      _setState(BleState.readingChallenge);
      final challenge = await challengeChar.read();

      // Note: The actual signing with the challenge should be done by the caller
      // before calling this method. The signedPresentation already includes
      // the challenge binding.

      // Subscribe to result before writing presentation
      _setState(BleState.awaitingResult);
      await resultChar.setNotifyValue(true);

      _resultSubscription = resultChar.onValueReceived.listen((value) {
        if (value.isNotEmpty) {
          final resultByte = value[0];
          final result = switch (resultByte) {
            0x01 => PresentationResult.granted,
            0x02 => PresentationResult.denied,
            _ => PresentationResult.error,
          };
          if (!completer.isCompleted) {
            completer.complete(result);
          }
        }
      });

      // Write presentation
      _setState(BleState.presenting);
      await presentationChar.write(
        signedPresentation,
        withoutResponse: false, // Need confirmation
      );

      // Wait for result with timeout
      final result = await completer.future.timeout(
        const Duration(seconds: 5),
        onTimeout: () => PresentationResult.timeout,
      );

      _setState(BleState.complete);
      _resultController.add(result);
      return result;
    } catch (e) {
      _setState(BleState.error);
      _resultController.add(PresentationResult.error);
      return PresentationResult.error;
    } finally {
      await _resultSubscription?.cancel();
      _resultSubscription = null;
    }
  }

  /// Read the challenge from the verifier.
  Future<Uint8List?> readChallenge() async {
    if (_connectedDevice == null) return null;

    try {
      final services = await _connectedDevice!.discoverServices();
      final service = services.firstWhere(
        (s) => s.uuid == BleUuids.serviceUuid,
      );
      final challengeChar = service.characteristics.firstWhere(
        (c) => c.uuid == BleUuids.challengeUuid,
      );

      final challenge = await challengeChar.read();
      return Uint8List.fromList(challenge);
    } catch (e) {
      return null;
    }
  }

  /// Disconnect from the current device.
  Future<void> disconnect() async {
    await _resultSubscription?.cancel();
    _resultSubscription = null;
    await _connectionSubscription?.cancel();
    _connectionSubscription = null;

    if (_connectedDevice != null) {
      await _connectedDevice!.disconnect();
      _connectedDevice = null;
    }

    _setState(BleState.idle);
  }

  void _handleDisconnection() {
    _connectedDevice = null;
    if (_currentState != BleState.complete && _currentState != BleState.idle) {
      _setState(BleState.error);
      _resultController.add(PresentationResult.error);
    }
  }

  /// Clean up resources.
  void dispose() {
    stopScanning();
    disconnect();
    _stateController.close();
    _resultController.close();
  }
}

/// Provider for BLE service.
final bleServiceProvider = Provider<BleService>((ref) {
  final service = BleService();
  ref.onDispose(() => service.dispose());
  return service;
});
