package dev.witwizard.radsho.vpn

import android.app.Activity
import android.content.Intent
import android.graphics.Color
import android.net.VpnService
import android.system.Os
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.annotation.Keep
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
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
    init {
        runCatching {
            Os.setenv("RADSHO_NATIVE_LIB_DIR", activity.applicationInfo.nativeLibraryDir, true)
        }
    }

    override fun load(webView: WebView) {
        runCatching {
            Os.setenv("RADSHO_NATIVE_LIB_DIR", activity.applicationInfo.nativeLibraryDir, true)
        }
        activity.runOnUiThread {
            runCatching {
                activity.window.statusBarColor = Color.parseColor("#0E1514")
                activity.window.navigationBarColor = Color.parseColor("#0E1514")
                ViewCompat.setOnApplyWindowInsetsListener(webView) { view, insets ->
                    val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
                    val density = view.resources.displayMetrics.density.coerceAtLeast(1f)
                    val topDp = (bars.top / density).toInt()
                    val bottomDp = (bars.bottom / density).toInt()
                    webView.evaluateJavascript(
                        "document.documentElement.style.setProperty('--android-inset-top', '${topDp}px');" +
                            "document.documentElement.style.setProperty('--android-inset-bottom', '${bottomDp}px');",
                        null
                    )
                    insets
                }
            }
        }
    }

    @Command
    fun startVpn(invoke: Invoke) {
        val args = invoke.parseArgs(StartArgs::class.java)
        RadshoVpnService.pending = args
        activity.runOnUiThread {
            runCatching {
                val consent = VpnService.prepare(activity)
                if (consent != null) {
                    startActivityForResult(invoke, consent, "onVpnPermission")
                } else {
                    launchService(RadshoVpnService.ACTION_START)
                    invoke.resolve()
                }
            }.onFailure { err ->
                invoke.reject(err.message ?: "vpn start failed")
            }
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
        activity.runOnUiThread {
            launchService(RadshoVpnService.ACTION_STOP)
            invoke.resolve()
        }
    }

    private fun launchService(action: String) {
        runCatching {
            val intent = Intent(activity, RadshoVpnService::class.java).setAction(action)
            activity.startService(intent)
        }
    }
}