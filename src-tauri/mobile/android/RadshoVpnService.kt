package dev.witwizard.radsho.vpn

import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.annotation.Keep
import hev.htproxy.TProxyService
import java.io.File

@Keep
class RadshoVpnService : VpnService() {
    companion object {
        const val ACTION_START = "dev.witwizard.radsho.vpn.START"
        const val ACTION_STOP = "dev.witwizard.radsho.vpn.STOP"

        @Volatile
        var pending: StartArgs? = null

        private val packageMap: Map<String, String> = mapOf(
            "Telegram" to "org.telegram.messenger",
            "Chrome" to "com.android.chrome",
            "Firefox" to "org.mozilla.firefox",
            "Spotify" to "com.spotify.music",
            "Steam" to "com.valvesoftware.android.steam.community",
            "WhatsApp" to "com.whatsapp",
            "Discord" to "com.discord",
        )
    }

    private var tunnel: ParcelFileDescriptor? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopTunnel()
            stopSelf()
            return START_NOT_STICKY
        }
        val args = pending ?: return START_NOT_STICKY
        startTunnel(args)
        return START_STICKY
    }

    override fun onDestroy() {
        stopTunnel()
        super.onDestroy()
    }

    override fun onRevoke() {
        stopTunnel()
        super.onRevoke()
    }

    private fun startTunnel(args: StartArgs) {
        stopTunnel()
        val builder = Builder()
            .setSession("Radsho")
            .setMtu(8500)
            .addAddress("172.19.0.1", 30)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("172.19.0.2")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            runCatching {
                builder.setMetered(false)
            }
        }
        if (args.allowlist) {
            args.apps.mapNotNull { packageMap[it] }.forEach { name ->
                runCatching { builder.addAllowedApplication(name) }
            }
        } else {
            runCatching { builder.addDisallowedApplication(packageName) }
        }
        val descriptor = builder.establish() ?: return
        tunnel = descriptor
        val config = File(filesDir, "tproxy.yml")
        config.writeText(
            """
            tunnel:
              mtu: 8500
              ipv4: 172.19.0.1
            socks5:
              port: ${args.socksPort}
              address: 127.0.0.1
              udp: 'tcp'
            mapdns:
              address: 172.19.0.2
              port: 53
              network: 198.18.0.0
              netmask: 255.254.0.0
              cache-size: 10000
            misc:
              task-stack-size: 81920
              log-level: warn
            """.trimIndent()
        )
        runCatching {
            TProxyService.TProxyStartService(config.absolutePath, descriptor.fd)
        }
    }

    private fun stopTunnel() {
        if (tunnel != null) {
            runCatching { TProxyService.TProxyStopService() }
            runCatching { tunnel?.close() }
            tunnel = null
        }
    }
}