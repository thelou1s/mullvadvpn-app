package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.support.v4.app.FragmentActivity

import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

class MainActivity : FragmentActivity() {
    private lateinit var extractDaemonJob: Job
    private lateinit var startDaemonJob: Job
    private lateinit var daemon: MullvadDaemon
    private lateinit var daemonProcess: Process
    private val daemonStarted = CompletableDeferred<Unit>()

    val ipcClient = MullvadIpcClient()

    var asyncRelayList: Deferred<RelayList> = fetchRelayList()
        private set
    val relayList: RelayList
        get() = runBlocking { asyncRelayList.await() }

    var asyncSettings: Deferred<Settings> = fetchSettings()
        private set

    val settings: Settings
        get() = runBlocking { asyncSettings.await() }

    var selectedRelayItem: RelayItem? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        daemon = MullvadDaemon()
        startDaemonJob = startDaemon()

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    override fun onDestroy() {
        asyncRelayList.cancel()
        asyncSettings.cancel()
        daemonStarted.cancel()
        extractDaemonJob.cancel()
        startDaemonJob.cancel()
        daemonProcess.destroy()

        super.onDestroy()
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LaunchFragment())
            commit()
        }
    }

    private fun startDaemon() = GlobalScope.launch(Dispatchers.Main) {
        extractDaemonJob = GlobalScope.launch(Dispatchers.Default) {
            daemon.extract(this@MainActivity)
        }

        extractDaemonJob.join()
        daemonProcess = daemon.run()
        daemonStarted.complete(Unit)
        restoreSelectedRelayListItem()
    }

    private fun fetchRelayList() = GlobalScope.async(Dispatchers.Default) {
        daemonStarted.await()
        RelayList(ipcClient.getRelayLocations())
    }

    private fun fetchSettings() = GlobalScope.async(Dispatchers.Default) {
        daemonStarted.await()
        ipcClient.getSettings()
    }

    private suspend fun restoreSelectedRelayListItem() {
        val relaySettings = asyncSettings.await().relaySettings

        when (relaySettings) {
            is RelaySettings.CustomTunnelEndpoint -> selectedRelayItem = null
            is RelaySettings.RelayConstraints -> {
                val location = relaySettings.location
                val relayList = asyncRelayList.await()

                selectedRelayItem = relayList.findItemForLocation(location)
            }
        }
    }
}
