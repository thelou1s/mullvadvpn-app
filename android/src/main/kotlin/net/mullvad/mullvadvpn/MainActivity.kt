package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.support.v4.app.FragmentActivity

import net.mullvad.mullvadvpn.relaylist.RelayList

class MainActivity : FragmentActivity() {
    private lateinit var extractDaemonJob: Job
    private lateinit var startDaemonJob: Job
    private lateinit var daemon: MullvadDaemon
    private lateinit var daemonProcess: Process

    val ipcClient = MullvadIpcClient()

    private var getRelayListJob: Deferred<RelayList>? = null
    val relayList: RelayList
        get() = runBlocking {
            getRelayListJob!!.await()
        }

    var selectedRelayItemCode: String? = null

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
        getRelayListJob?.cancel()
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

        getRelayListJob = fetchRelayList()
    }

    private fun fetchRelayList() = GlobalScope.async(Dispatchers.Default) {
        RelayList(ipcClient.getRelayLocations())
    }
}
