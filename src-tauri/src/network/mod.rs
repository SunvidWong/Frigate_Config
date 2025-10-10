// Network module for camera discovery and network utilities

pub mod camera_discovery;

pub use camera_discovery::{
    DiscoveredCamera,
    ScanConfig,
    scan_network,
    get_local_ip,
    guess_network_range,
};
