// Network module for camera discovery and network utilities

pub mod camera_discovery;

pub use camera_discovery::{
    get_local_ip, guess_network_range, scan_network, DiscoveredCamera, ScanConfig,
};
