import NetworkExtension
import Tauri
import UIKit
import WebKit

struct StartArgs: Decodable {
    let socksPort: Int
    let httpPort: Int
    let dns: String
    let allowlist: Bool
    let apps: [String]
    let direct: [String]
}

class RadshoVpnPlugin: Plugin {
    @objc public func startVpn(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StartArgs.self)
        NETunnelProviderManager.loadAllFromPreferences { managers, loadError in
            if let loadError = loadError {
                invoke.reject(loadError.localizedDescription)
                return
            }
            let manager = managers?.first ?? NETunnelProviderManager()
            let proto = NETunnelProviderProtocol()
            proto.providerBundleIdentifier = "dev.witwizard.radsho.tunnel"
            proto.serverAddress = "Radsho"
            proto.providerConfiguration = [
                "socksPort": args.socksPort,
                "httpPort": args.httpPort,
                "dns": args.dns,
            ]
            manager.protocolConfiguration = proto
            manager.localizedDescription = "Radsho"
            manager.isEnabled = true
            manager.saveToPreferences { saveError in
                if let saveError = saveError {
                    invoke.reject(saveError.localizedDescription)
                    return
                }
                manager.loadFromPreferences { reloadError in
                    if let reloadError = reloadError {
                        invoke.reject(reloadError.localizedDescription)
                        return
                    }
                    do {
                        try manager.connection.startVPNTunnel()
                        invoke.resolve()
                    } catch {
                        invoke.reject(error.localizedDescription)
                    }
                }
            }
        }
    }

    @objc public func stopVpn(_ invoke: Invoke) throws {
        NETunnelProviderManager.loadAllFromPreferences { managers, _ in
            managers?.first?.connection.stopVPNTunnel()
            invoke.resolve()
        }
    }
}

@_cdecl("init_plugin_radsho_vpn")
func initPlugin() -> Plugin {
    return RadshoVpnPlugin()
}
