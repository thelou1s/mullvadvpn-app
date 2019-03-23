package net.mullvad.mullvadvpn

import java.io.File
import java.io.InputStream
import java.io.OutputStream

class MullvadIpcClient {
    init {
        System.loadLibrary("mullvad_jni")
        startLogging()
    }

    external fun getAccountData(accountToken: String): AccountData?
    external fun getCurrentVersion(): String
    external fun setAccount(accountToken: String?)

    private external fun startLogging()
}
