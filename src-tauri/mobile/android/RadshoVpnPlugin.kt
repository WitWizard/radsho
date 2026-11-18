package dev.witwizard.radsho.vpn

import android.app.Activity
import android.content.Intent
import android.graphics.Color
import android.net.VpnService
import android.system.Os
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.annotation.Keep
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@Keep
@InvokeArg
class StartArgs {
    var socksPort: Int = 1821
    var httpPort: Int = 1822
    var dns: String = "1.1.1.1"
    var allowlist: Boolean = false
    var apps: Array<String> = emptyArray()
    var direct: Array<String> = emptyArray()
}

@Keep
@TauriPlugin
class RadshoVpnPlugin(private val activity: Activity) : Plugin(activity) {
    override fun load(webView: WebView) {
        runCatching {
            Os.setenv("RADSHO_NATIVE_LIB_DIR", activity.applicationInfo.nativeLibraryDir, true)
        }
        activity.runOnUiThread {
            runCatching {
                @Suppress("DEPRECATION")
                activity.window.statusBarColor = Color.parseColor("#0E1514")
                @Suppress("DEPRECATION")
                activity.window.navigationBarColor = Color.parseColor("#0E1514")
                webView.setOnApplyWindowInsetsListener { view, insets ->
                    @Suppress("DEPRECATION")
                    view.setPadding(
                        insets.systemWindowInsetLeft,
                        insets.systemWindowInsetTop,
                        insets.systemWindowInsetRight,
                        insets.systemWindowInsetBottom
                    )
                    @Suppress("DEPRECATION")
                    insets.consumeSystemWindowInsets()
                }
                webView.requestApplyInsets()
            }
        }
    }

    @Command
    fun startVpn(invoke: Invoke) {
        val args = invoke.parseArgs(StartArgs::class.java)
        RadshoVpnService.pending = args
        val consent = VpnService.prepare(activity)
        if (consent != null) {
            startActivityForResult(invoke, consent, "onVpnPermission")
        } else {
            launchService(RadshoVpnService.ACTION_START)
            invoke.resolve()
        }
    }

    @ActivityCallback
    fun onVpnPermission(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode == Activity.RESULT_OK) {
            launchService(RadshoVpnService.ACTION_START)
            invoke.resolve()
        } else {
            invoke.reject("vpn permission denied")
        }
    }

    @Command
    fun stopVpn(invoke: Invoke) {
        launchService(RadshoVpnService.ACTION_STOP)
        invoke.resolve()
    }

    private fun launchService(action: String) {
        val intent = Intent(activity, RadshoVpnService::class.java).setAction(action)
        activity.startService(intent)
    }
}