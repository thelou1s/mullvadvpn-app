package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus

    private lateinit var ipcClient: MullvadIpcClient

    private var updateViewJob: Job? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        ipcClient = (context as MainActivity).ipcClient
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        view.findViewById<Button>(R.id.switch_location).setOnClickListener {
            openSwitchLocationScreen()
        }

        headerBar = HeaderBar(view, context!!)
        notificationBanner = NotificationBanner(view)
        status = ConnectionStatus(view, context!!)

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { ipcClient.connect() }
            onCancel = { ipcClient.disconnect() }
            onDisconnect = { ipcClient.disconnect() }
        }

        ipcClient.onTunnelStateChange = { state -> updateViewJob = updateView(state) }

        return view
    }

    override fun onDestroyView() {
        updateViewJob?.cancel()
        ipcClient.onTunnelStateChange = null
    }

    private fun updateView(state: TunnelStateTransition) = GlobalScope.launch(Dispatchers.Main) {
        actionButton.state = state
        headerBar.setState(state)
        notificationBanner.setState(state)
        status.setState(state)
    }

    private fun openSwitchLocationScreen() {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SelectLocationFragment())
            addToBackStack(null)
            commit()
        }
    }
}
