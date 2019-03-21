package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.support.v4.app.FragmentActivity

class MainActivity : FragmentActivity() {
    private lateinit var extractDaemonJob: Job
    private lateinit var startDaemonJob: Job
    private lateinit var daemon: MullvadDaemon
    private lateinit var daemonProcess: Process

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
        extractDaemonJob.cancel()
        startDaemonJob.cancel()
        daemonProcess.destroy()

        super.onDestroy()
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun startDaemon() = GlobalScope.launch(Dispatchers.Main) {
        extractDaemonJob = GlobalScope.launch(Dispatchers.Default) {
            daemon.extract(this@MainActivity)
        }

        extractDaemonJob.join()
        daemonProcess = daemon.run()
    }
}
