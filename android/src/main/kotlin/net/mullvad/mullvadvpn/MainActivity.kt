package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.support.v4.app.FragmentActivity

class MainActivity : FragmentActivity() {
    private lateinit var extractDaemonJob: Job

    var selectedRelayItemCode: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        extractDaemonJob = GlobalScope.launch(Dispatchers.Default) {
            MullvadDaemon().extract(this@MainActivity)
        }

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    override fun onDestroy() {
        if (extractDaemonJob.isActive) {
            extractDaemonJob.cancel()
        }

        super.onDestroy()
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LoginFragment())
            commit()
        }
    }
}
