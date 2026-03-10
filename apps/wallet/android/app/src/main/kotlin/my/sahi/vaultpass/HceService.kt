package my.sahi.vaultpass

import android.nfc.cardemulation.HostApduService
import android.os.Bundle

/**
 * Host Card Emulation service for NFC credential presentation.
 *
 * Emulates an NFC tag that verifiers can read.
 * Payload is CBOR-encoded SD-JWT VP.
 */
class HceService : HostApduService() {

    companion object {
        // APDU command bytes
        private const val SELECT_INS: Byte = 0xA4.toByte()
        private const val READ_INS: Byte = 0xB0.toByte()

        // Status words
        private val SW_SUCCESS = byteArrayOf(0x90.toByte(), 0x00)
        private val SW_FILE_NOT_FOUND = byteArrayOf(0x6A.toByte(), 0x82.toByte())
        private val SW_UNKNOWN = byteArrayOf(0x6F.toByte(), 0x00)

        // VaultPass AID
        private val VAULTPASS_AID = byteArrayOf(
            0xF0.toByte(), 0x53, 0x41, 0x48, 0x49, // "SAHI"
            0x56, 0x41, 0x50 // "VAP"
        )

        // Current presentation payload (set by Flutter)
        @Volatile
        var pendingPayload: ByteArray? = null
    }

    private var isSelected = false

    override fun processCommandApdu(commandApdu: ByteArray?, extras: Bundle?): ByteArray {
        if (commandApdu == null || commandApdu.size < 4) {
            return SW_UNKNOWN
        }

        val ins = commandApdu[1]

        return when (ins) {
            SELECT_INS -> handleSelect(commandApdu)
            READ_INS -> handleRead()
            else -> SW_UNKNOWN
        }
    }

    private fun handleSelect(apdu: ByteArray): ByteArray {
        // Check if selecting our AID
        if (apdu.size < 5 + VAULTPASS_AID.size) {
            return SW_FILE_NOT_FOUND
        }

        val aid = apdu.copyOfRange(5, 5 + VAULTPASS_AID.size)
        if (!aid.contentEquals(VAULTPASS_AID)) {
            return SW_FILE_NOT_FOUND
        }

        isSelected = true
        return SW_SUCCESS
    }

    private fun handleRead(): ByteArray {
        if (!isSelected) {
            return SW_FILE_NOT_FOUND
        }

        val payload = pendingPayload
        if (payload == null) {
            return SW_FILE_NOT_FOUND
        }

        // Clear payload after read (one-time use)
        pendingPayload = null
        isSelected = false

        // Return payload with success status
        return payload + SW_SUCCESS
    }

    override fun onDeactivated(reason: Int) {
        isSelected = false
    }
}
