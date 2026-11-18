package dev.witwizard.radsho.vpn

import android.app.Activity
import android.content.Intent
import android.graphics.Color
import android.net.VpnService
import android.system.Os
import android.view.View
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.annotation.Keep
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
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
                val window = activity.window
                WindowCompat.setDecorFitsSystemWindows(window, false)
                window.statusBarColor = Color.parseColor("#0E1514")
                window.navigationBarColor = Color.parseColor("#0E1514")
                val controller = WindowInsetsControllerCompat(window, window.decorView)
                controller.isAppearanceLightStatusBars = false
                controller.isAppearanceLightNavigationBars = false

                val contentView = window.decorView.findViewById<View>(android.R.id.content)
                if (contentView != null) {
                    contentView.setBackgroundColor(Color.parseColor("#0E1514"))
                    ViewCompat.setOnApplyWindowInsetsListener(contentView) { view, windowInsets ->
                        val insets = windowInsets.getInsets(
                            WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
                        )
                        view.setPadding(insets.left, insets.top, insets.right, insets.bottom)
                        WindowInsetsCompat.CONSUMED
                    }
                    ViewCompat.requestApplyInsets(contentView)
                }
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