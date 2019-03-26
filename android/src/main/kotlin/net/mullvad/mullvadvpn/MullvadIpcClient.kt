package net.mullvad.mullvadvpn

import java.io.File
import java.io.InputStream
import java.io.OutputStream

import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings

class MullvadIpcClient {
    init {
        System.loadLibrary("mullvad_jni")
        startLogging()
        loadClasses()
    }

    external fun getAccountData(accountToken: String): AccountData?
    external fun getCurrentVersion(): String
    external fun getRelayLocations(): RelayList
    external fun getSettings(): Settings
    external fun setAccount(accountToken: String?)
    external fun updateRelaySettings(update: RelaySettingsUpdate)

    private external fun startLogging()
    private external fun loadClasses()
}
