// Camera Discovery Module
// Network scanning for IP cameras

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::time::timeout;

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// 接口名称
    pub name: String,
    /// IP 地址
    pub ip: String,
    /// 子网掩码
    pub subnet: String,
    /// 是否为默认接口
    pub is_default: bool,
}

/// Discovered camera device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCamera {
    /// IP address
    pub ip: String,
    /// Open ports found
    pub ports: Vec<u16>,
    /// Device type hint (based on ports)
    pub device_type: String,
    /// Hostname (if resolvable)
    pub hostname: Option<String>,
    /// MAC address (if discoverable)
    pub mac_address: Option<String>,
    /// Possible RTSP URLs
    pub rtsp_urls: Vec<String>,
    /// Possible HTTP URLs
    pub http_urls: Vec<String>,
    /// Last seen timestamp
    pub last_seen: i64,
}

/// Network scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Network range to scan (e.g., "192.168.1.0/24")
    pub network_range: String,
    /// Ports to scan
    pub ports: Vec<u16>,
    /// Timeout per port (milliseconds)
    pub timeout_ms: u64,
    /// Concurrent scans
    pub concurrency: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            network_range: "192.168.1.0/24".to_string(),
            ports: vec![554, 80, 8000, 8080, 8554, 8888],
            timeout_ms: 1000,
            concurrency: 50,
        }
    }
}

/// Common camera ports
pub const RTSP_PORTS: &[u16] = &[554, 8554];
pub const HTTP_PORTS: &[u16] = &[80, 8000, 8080, 8888];
pub const ONVIF_PORT: u16 = 3702;

impl DiscoveredCamera {
    /// Create new discovered camera
    pub fn new(ip: String) -> Self {
        Self {
            ip,
            ports: Vec::new(),
            device_type: "Unknown".to_string(),
            hostname: None,
            mac_address: None,
            rtsp_urls: Vec::new(),
            http_urls: Vec::new(),
            last_seen: chrono::Utc::now().timestamp(),
        }
    }

    /// Add discovered port
    pub fn add_port(&mut self, port: u16) {
        if !self.ports.contains(&port) {
            self.ports.push(port);
            self.ports.sort();
            self.update_device_type();
            self.generate_urls();
        }
    }

    /// Update device type based on open ports
    fn update_device_type(&mut self) {
        let has_rtsp = self.ports.iter().any(|p| RTSP_PORTS.contains(p));
        let has_http = self.ports.iter().any(|p| HTTP_PORTS.contains(p));
        let has_onvif = self.ports.contains(&ONVIF_PORT);

        self.device_type = match (has_rtsp, has_http, has_onvif) {
            (true, true, true) => "ONVIF Camera (RTSP + HTTP)".to_string(),
            (true, true, false) => "IP Camera (RTSP + HTTP)".to_string(),
            (true, false, _) => "RTSP Camera".to_string(),
            (false, true, _) => "HTTP Camera".to_string(),
            _ => "Network Device".to_string(),
        };
    }

    /// Generate possible URLs
    fn generate_urls(&mut self) {
        // Generate RTSP URLs
        self.rtsp_urls.clear();
        for port in &self.ports {
            if RTSP_PORTS.contains(port) {
                self.rtsp_urls.push(format!("rtsp://{}:{}/", self.ip, port));
                // Common RTSP paths
                self.rtsp_urls
                    .push(format!("rtsp://{}:{}/stream1", self.ip, port));
                self.rtsp_urls
                    .push(format!("rtsp://{}:{}/live/main", self.ip, port));
                self.rtsp_urls
                    .push(format!("rtsp://{}:{}/h264", self.ip, port));
            }
        }

        // Generate HTTP URLs
        self.http_urls.clear();
        for port in &self.ports {
            if HTTP_PORTS.contains(port) {
                self.http_urls.push(format!("http://{}:{}/", self.ip, port));
            }
        }
    }
}

/// Parse network range to IP list
pub fn parse_network_range(range: &str) -> Result<Vec<IpAddr>, String> {
    // Simple CIDR parsing (e.g., "192.168.1.0/24")
    if let Some((network, prefix)) = range.split_once('/') {
        let network_parts: Vec<&str> = network.split('.').collect();
        if network_parts.len() != 4 {
            return Err("Invalid network format".to_string());
        }

        let prefix_len: u8 = prefix.parse().map_err(|_| "Invalid prefix length")?;
        if prefix_len > 32 {
            return Err("Prefix length must be <= 32".to_string());
        }

        let base_octets: Result<Vec<u8>, _> =
            network_parts.iter().map(|s| s.parse::<u8>()).collect();

        let base_octets = base_octets.map_err(|_| "Invalid IP address")?;

        let mut ips = Vec::new();
        let host_bits = 32 - prefix_len;
        let num_hosts = 2u32.pow(host_bits as u32);

        // Generate IP addresses in the range
        for i in 1..num_hosts - 1 {
            // Skip network and broadcast
            let ip = IpAddr::from([
                base_octets[0],
                base_octets[1],
                base_octets[2],
                (base_octets[3] as u32 + i) as u8,
            ]);
            ips.push(ip);
        }

        Ok(ips)
    } else {
        Err("Invalid CIDR notation".to_string())
    }
}

/// Check if a port is open on a host
pub async fn check_port(ip: IpAddr, port: u16, timeout_ms: u64) -> bool {
    let addr = SocketAddr::new(ip, port);
    let duration = Duration::from_millis(timeout_ms);

    match timeout(duration, TokioTcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

/// Scan a single IP for open ports
pub async fn scan_ip(ip: IpAddr, ports: &[u16], timeout_ms: u64) -> Option<DiscoveredCamera> {
    let ip_str = ip.to_string();
    let mut camera = DiscoveredCamera::new(ip_str.clone());
    let mut found_any = false;

    for &port in ports {
        if check_port(ip, port, timeout_ms).await {
            camera.add_port(port);
            found_any = true;
        }
    }

    if found_any {
        // Try to resolve hostname
        camera.hostname = resolve_hostname(&ip).await;
        Some(camera)
    } else {
        None
    }
}

/// Resolve IP to hostname
async fn resolve_hostname(_ip: &IpAddr) -> Option<String> {
    // Reverse DNS lookup disabled for now
    // Would require additional dependencies (dns_lookup crate)
    // This is a placeholder for future implementation
    None
}

/// Scan network for cameras
pub async fn scan_network(config: ScanConfig) -> Result<Vec<DiscoveredCamera>, String> {
    let ips = parse_network_range(&config.network_range)?;
    let mut cameras = Vec::new();

    // Use tokio tasks for concurrent scanning
    let mut tasks = Vec::new();

    for ip in ips {
        let ports = config.ports.clone();
        let timeout = config.timeout_ms;

        let task = tokio::spawn(async move { scan_ip(ip, &ports, timeout).await });

        tasks.push(task);

        // Limit concurrency
        if tasks.len() >= config.concurrency {
            // Wait for some to complete
            let results = futures::future::join_all(tasks).await;
            for result in results {
                if let Ok(Some(camera)) = result {
                    cameras.push(camera);
                }
            }
            tasks = Vec::new();
        }
    }

    // Wait for remaining tasks
    let results = futures::future::join_all(tasks).await;
    for result in results {
        if let Ok(Some(camera)) = result {
            cameras.push(camera);
        }
    }

    Ok(cameras)
}

/// 获取所有物理网络接口
pub fn get_all_network_interfaces() -> Vec<NetworkInterface> {
    use if_addrs::{get_if_addrs, IfAddr};

    let mut interfaces = Vec::new();
    let mut default_ip = None;

    // 先获取默认 IP
    if let Some(ip) = get_local_ip() {
        default_ip = Some(ip);
    }

    // 获取所有网络接口
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            // 跳过回环接口
            if iface.is_loopback() {
                continue;
            }

            // 跳过虚拟接口
            let name = iface.name.to_lowercase();
            if name.contains("docker")
                || name.contains("vbox")
                || name.contains("vmware")
                || name.contains("veth")
                || name.contains("virbr")
                || name.contains("tun")
                || name.contains("tap")
            {
                continue;
            }

            // 只处理 IPv4 地址
            if let IfAddr::V4(v4_addr) = iface.addr {
                let ip = v4_addr.ip;
                let octets = ip.octets();

                // 只处理私有 IP 地址
                let is_private = match octets[0] {
                    192 if octets[1] == 168 => true,
                    10 => true,
                    172 if (16..=31).contains(&octets[1]) => true,
                    _ => false,
                };

                if is_private {
                    let ip_str = ip.to_string();
                    let subnet = format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]);
                    let is_default = default_ip.as_ref() == Some(&ip_str);

                    interfaces.push(NetworkInterface {
                        name: iface.name.clone(),
                        ip: ip_str,
                        subnet,
                        is_default,
                    });
                }
            }
        }
    }

    // 如果没有找到任何接口，添加一个默认的
    if interfaces.is_empty() {
        interfaces.push(NetworkInterface {
            name: "default".to_string(),
            ip: "192.168.1.100".to_string(),
            subnet: "192.168.1.0/24".to_string(),
            is_default: true,
        });
    }

    interfaces
}

/// Get local network interface IP (excluding virtual interfaces)
pub fn get_local_ip() -> Option<String> {
    use if_addrs::{get_if_addrs, IfAddr};

    // Get all network interfaces
    if let Ok(interfaces) = get_if_addrs() {
        for iface in interfaces {
            // Skip loopback and virtual interfaces
            if iface.is_loopback() {
                continue;
            }

            // Skip known virtual interface names
            let name = iface.name.to_lowercase();
            if name.contains("docker")
                || name.contains("vbox")
                || name.contains("vmware")
                || name.contains("veth")
                || name.contains("virbr")
                || name.contains("tun")
                || name.contains("tap")
            {
                continue;
            }

            // Get IPv4 address from physical interface
            if let IfAddr::V4(v4_addr) = iface.addr {
                let ip = v4_addr.ip;

                // Skip non-private IP ranges
                // Accept: 192.168.x.x, 10.x.x.x, 172.16-31.x.x
                let octets = ip.octets();
                let is_private = match octets[0] {
                    192 if octets[1] == 168 => true,
                    10 => true,
                    172 if (16..=31).contains(&octets[1]) => true,
                    _ => false,
                };

                if is_private {
                    return Some(ip.to_string());
                }
            }
        }
    }

    // Fallback: use UDP socket method
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Guess network range from local IP
pub fn guess_network_range() -> String {
    if let Some(local_ip) = get_local_ip() {
        let parts: Vec<&str> = local_ip.split('.').collect();
        if parts.len() == 4 {
            return format!("{}.{}.{}.0/24", parts[0], parts[1], parts[2]);
        }
    }
    "192.168.1.0/24".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_network_range() {
        let ips = parse_network_range("192.168.1.0/24").unwrap();
        assert_eq!(ips.len(), 254); // 256 - 2 (network and broadcast)
        assert_eq!(ips[0].to_string(), "192.168.1.1");
        assert_eq!(ips[253].to_string(), "192.168.1.254");
    }

    #[test]
    fn test_camera_device_type() {
        let mut camera = DiscoveredCamera::new("192.168.1.100".to_string());
        camera.add_port(554);
        assert_eq!(camera.device_type, "RTSP Camera");

        camera.add_port(80);
        assert_eq!(camera.device_type, "IP Camera (RTSP + HTTP)");
    }

    #[test]
    fn test_guess_network_range() {
        let range = guess_network_range();
        assert!(range.ends_with(".0/24"));
    }
}
