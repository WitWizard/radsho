package hev.htproxy

import androidx.annotation.Keep

@Keep
object TProxyService {
    init {
        runCatching {
            System.loadLibrary("hev-socks5-tunnel")
        }
    }

    @Keep
    @JvmStatic
    external fun TProxyStartService(configPath: String, fd: Int)

    @Keep
    @JvmStatic
    external fun TProxyStopService()

    @Keep
    @JvmStatic
    external fun TProxyGetStats(): LongArray?
}