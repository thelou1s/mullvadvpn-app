package net.mullvad.mullvadvpn

import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.lang.ProcessBuilder.Redirect

import android.content.Context

private const val API_ROOT_CA_FILE = "api_root_ca.pem"
private const val MULLVAD_DAEMON_EXE = "mullvad-daemon"

private const val API_ROOT_CA_PATH = "/data/data/net.mullvad.mullvadvpn/api_root_ca.pem"
private const val MULLVAD_DAEMON_PATH = "/data/data/net.mullvad.mullvadvpn/mullvad-daemon"

class MullvadDaemon {
    fun extract(context: Context) {
        if (!File(API_ROOT_CA_PATH).exists()) {
            extractFile(context, API_ROOT_CA_FILE, API_ROOT_CA_PATH)
        }

        if (!File(MULLVAD_DAEMON_PATH).canExecute()) {
            extractFile(context, MULLVAD_DAEMON_EXE, MULLVAD_DAEMON_PATH)
            Runtime.getRuntime().exec("/system/bin/chmod 750 $MULLVAD_DAEMON_PATH").waitFor()
        }
    }

    fun run() = ProcessBuilder(MULLVAD_DAEMON_PATH, "-vvv")
        .redirectErrorStream(true)
        .redirectOutput(Redirect.appendTo(File("/dev/null")))
        .start()

    private fun extractFile(context: Context, asset: String, destination: String) {
        val destinationStream = FileOutputStream(destination)

        context
            .assets
            .open(asset)
            .copyTo(destinationStream)

        destinationStream.close()
    }
}
