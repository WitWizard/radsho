import Foundation
import HevSocks5Tunnel
import NetworkExtension

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var worker: Thread?

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let configuration = (protocolConfiguration as? NETunnelProviderProtocol)?.providerConfiguration ?? [:]
        let socksPort = configuration["socksPort"] as? Int ?? 1821
        let dns = configuration["dns"] as? String ?? "1.1.1.1"

        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        let ipv4 = NEIPv4Settings(addresses: ["198.18.0.1"], subnetMasks: ["255.255.255.255"])
        ipv4.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4
        settings.dnsSettings = NEDNSSettings(servers: [dns])
        settings.mtu = 8500

        setTunnelNetworkSettings(settings) { [weak self] error in
            guard let self = self else { return }
            if let error = error {
                completionHandler(error)
                return
            }
            guard let descriptor = self.packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 else {
                completionHandler(NSError(domain: "dev.witwizard.radsho", code: 1, userInfo: [NSLocalizedDescriptionKey: "tunnel descriptor unavailable"]))
                return
            }
            let yaml = """
            tunnel:
              mtu: 8500
              ipv4: 198.18.0.1
            socks5:
              port: \(socksPort)
              address: 127.0.0.1
              udp: udp
            misc:
              log-level: warn
            """
            let thread = Thread {
                let bytes = Array(yaml.utf8)
                bytes.withUnsafeBufferPointer { buffer in
                    _ = hev_socks5_tunnel_main_from_str(buffer.baseAddress, UInt32(buffer.count), descriptor)
                }
            }
            thread.name = "radsho.tun2socks"
            thread.start()
            self.worker = thread
            completionHandler(nil)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        hev_socks5_tunnel_quit()
        worker = nil
        completionHandler()
    }
}
