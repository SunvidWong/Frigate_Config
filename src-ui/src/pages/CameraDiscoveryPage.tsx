import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Camera, Wifi, Play, RefreshCw, AlertCircle, Check, ExternalLink } from 'lucide-react';
import { safeInvoke, isTauriEnvironment } from '../utils/tauri';

interface NetworkInterface {
  name: string;
  ip: string;
  subnet: string;
  is_default: boolean;
}

interface DiscoveredCamera {
  ip: string;
  ports: number[];
  device_type: string;
  hostname?: string | null;
  mac_address?: string | null;
  rtsp_urls: string[];
  http_urls: string[];
  last_seen: number;
}

interface CameraBrand {
  name: string;
  paths: string[];
  defaultPort: number;
}

// localStorage keys for persistence
const SCAN_RESULTS_KEY = 'camera_scan_results';
const SCAN_TIMESTAMP_KEY = 'camera_scan_timestamp';
const SCAN_NETWORK_KEY = 'camera_scan_network';

const CAMERA_BRANDS: CameraBrand[] = [
  {
    name: '海康威视 (Hikvision)',
    paths: [
      '/Streaming/Channels/101',
      '/Streaming/Channels/102',
      '/h264/ch1/main/av_stream',
      '/h264/ch1/sub/av_stream',
    ],
    defaultPort: 554,
  },
  {
    name: '大华 (Dahua)',
    paths: [
      '/cam/realmonitor?channel=1&subtype=0',
      '/cam/realmonitor?channel=1&subtype=1',
      '/live/ch00_0',
      '/live/ch00_1',
    ],
    defaultPort: 554,
  },
  {
    name: 'TP-Link',
    paths: [
      '/stream1',
      '/stream2',
      '/h264',
      '/live/main',
    ],
    defaultPort: 554,
  },
  {
    name: '小米 (Xiaomi)',
    paths: [
      '/live/ch00_0',
      '/live/ch00_1',
    ],
    defaultPort: 8554,
  },
  {
    name: '萤石 (EZVIZ)',
    paths: [
      '/h264/ch1/main/av_stream',
      '/h264/ch1/sub/av_stream',
    ],
    defaultPort: 554,
  },
  {
    name: 'Reolink',
    paths: [
      '/h264Preview_01_main',
      '/h264Preview_01_sub',
    ],
    defaultPort: 554,
  },
  {
    name: '通用 ONVIF',
    paths: [
      '/onvif1',
      '/onvif/profile1',
      '/stream1',
    ],
    defaultPort: 554,
  },
  {
    name: '自定义',
    paths: [],
    defaultPort: 554,
  },
];

const CameraDiscoveryPage: React.FC = () => {
  const navigate = useNavigate();
  const [networkRange, setNetworkRange] = useState('');
  const [customPorts, setCustomPorts] = useState('554,80,8000,8080,8554');
  const [isScanning, setIsScanning] = useState(false);
  const [cameras, setCameras] = useState<DiscoveredCamera[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [scanProgress, setScanProgress] = useState(0);
  const [selectedBrand, setSelectedBrand] = useState<{ [key: string]: number }>({});
  const [customPath, setCustomPath] = useState<{ [key: string]: string }>({});
  const [isTauri, setIsTauri] = useState(false);
  const [networkInterfaces, setNetworkInterfaces] = useState<NetworkInterface[]>([]);
  const [selectedInterface, setSelectedInterface] = useState<NetworkInterface | null>(null);
  const [lastScanTime, setLastScanTime] = useState<Date | null>(null);

  // 保存扫描结果到 localStorage
  const saveScanResults = (results: DiscoveredCamera[]) => {
    const timestamp = new Date();
    localStorage.setItem(SCAN_RESULTS_KEY, JSON.stringify(results));
    localStorage.setItem(SCAN_TIMESTAMP_KEY, timestamp.toISOString());
    localStorage.setItem(SCAN_NETWORK_KEY, networkRange);
    setLastScanTime(timestamp);
  };

  // 从 localStorage 加载扫描结果
  const loadScanResults = () => {
    const savedResults = localStorage.getItem(SCAN_RESULTS_KEY);
    const savedTimestamp = localStorage.getItem(SCAN_TIMESTAMP_KEY);
    const savedNetwork = localStorage.getItem(SCAN_NETWORK_KEY);

    if (savedResults && savedTimestamp) {
      const results = JSON.parse(savedResults) as DiscoveredCamera[];
      const timestamp = new Date(savedTimestamp);

      // 只加载24小时内的结果
      const hoursSinceLastScan = (Date.now() - timestamp.getTime()) / (1000 * 60 * 60);
      if (hoursSinceLastScan < 24) {
        setCameras(results);
        setLastScanTime(timestamp);
        if (savedNetwork) {
          setNetworkRange(savedNetwork);
        }
        return true;
      }
    }
    return false;
  };

  // 清除扫描结果
  const clearScanResults = () => {
    if (window.confirm('确定要清除所有扫描结果吗？')) {
      localStorage.removeItem(SCAN_RESULTS_KEY);
      localStorage.removeItem(SCAN_TIMESTAMP_KEY);
      localStorage.removeItem(SCAN_NETWORK_KEY);
      setCameras([]);
      setLastScanTime(null);
      setError(null);
    }
  };

  // Check Tauri environment on mount
  useEffect(() => {
    setIsTauri(isTauriEnvironment());
  }, []);

  // Load network interfaces and saved results on mount
  useEffect(() => {
    if (isTauri) {
      loadNetworkInterfaces();
    } else {
      setNetworkRange('192.168.1.0/24');
    }

    // 尝试加载之前的扫描结果
    loadScanResults();
  }, [isTauri]);

  // 获取所有网络接口
  const loadNetworkInterfaces = async () => {
    try {
      const interfaces = await safeInvoke<NetworkInterface[]>('get_network_interfaces');
      setNetworkInterfaces(interfaces);

      // 自动选择默认接口
      const defaultInterface = interfaces.find(i => i.is_default);
      if (defaultInterface) {
        setSelectedInterface(defaultInterface);
        setNetworkRange(defaultInterface.subnet);
      } else if (interfaces.length > 0) {
        setSelectedInterface(interfaces[0]);
        setNetworkRange(interfaces[0].subnet);
      }
    } catch (err) {
      console.error('Failed to get network interfaces:', err);
      // 回退到旧的方式
      loadNetworkRange();
    }
  };

  const loadNetworkRange = async () => {
    try {
      const range = await safeInvoke<string>('guess_network_range_command');
      setNetworkRange(range);
    } catch (err) {
      console.error('Failed to guess network range:', err);
      setNetworkRange('192.168.1.0/24');
    }
  };

  // 切换网络接口时更新网络范围
  const handleInterfaceChange = (interfaceName: string) => {
    const selected = networkInterfaces.find(i => i.name === interfaceName);
    if (selected) {
      setSelectedInterface(selected);
      setNetworkRange(selected.subnet);
    }
  };

  const handleQuickScan = async () => {
    setIsScanning(true);
    setError(null);
    setScanProgress(0);
    setCameras([]);

    try {
      // Ensure we have the network range before scanning
      if (!networkRange) {
        await loadNetworkRange();
      }

      // Simulate progress
      const progressInterval = setInterval(() => {
        setScanProgress(prev => Math.min(prev + 10, 90));
      }, 500);

      // Use detected network range for quick scan
      const ports = [554, 80, 8000, 8080, 8554, 8888];
      const discovered = await safeInvoke<DiscoveredCamera[]>('scan_for_cameras', {
        networkRange: networkRange || '192.168.1.0/24',
        ports,
        timeoutMs: 1000,
      });

      clearInterval(progressInterval);
      setScanProgress(100);
      setCameras(discovered);

      // 保存扫描结果
      if (discovered.length > 0) {
        saveScanResults(discovered);
      }

      if (discovered.length === 0) {
        setError('未发现摄像头。请检查网络连接或尝试自定义扫描。');
      }
    } catch (err) {
      setError(`扫描失败: ${err}`);
    } finally {
      setIsScanning(false);
    }
  };

  const handleCustomScan = async () => {
    setIsScanning(true);
    setError(null);
    setScanProgress(0);
    setCameras([]);

    try {
      const ports = customPorts.split(',').map(p => parseInt(p.trim())).filter(p => !isNaN(p));

      const progressInterval = setInterval(() => {
        setScanProgress(prev => Math.min(prev + 10, 90));
      }, 500);

      const discovered = await safeInvoke<DiscoveredCamera[]>('scan_for_cameras', {
        networkRange,
        ports,
        timeoutMs: 1000,
      });

      clearInterval(progressInterval);
      setScanProgress(100);
      setCameras(discovered);

      // 保存扫描结果
      if (discovered.length > 0) {
        saveScanResults(discovered);
      }

      if (discovered.length === 0) {
        setError('未发现摄像头。请检查网络范围和端口设置。');
      }
    } catch (err) {
      setError(`扫描失败: ${err}`);
    } finally {
      setIsScanning(false);
    }
  };

  const getDeviceIcon = (deviceType: string) => {
    if (deviceType.includes('ONVIF')) {
      return <Camera className="w-6 h-6 text-blue-500" />;
    } else if (deviceType.includes('RTSP')) {
      return <Camera className="w-6 h-6 text-green-500" />;
    } else if (deviceType.includes('HTTP')) {
      return <Camera className="w-6 h-6 text-orange-500" />;
    }
    return <Wifi className="w-6 h-6 text-gray-500" />;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const generateRtspUrl = (cameraIp: string, cameraIndex: string) => {
    const brandIndex = selectedBrand[cameraIndex] || 0;
    const brand = CAMERA_BRANDS[brandIndex];

    if (brand.name === '自定义') {
      const path = customPath[cameraIndex] || '/';
      return `rtsp://${cameraIp}:${brand.defaultPort}${path}`;
    }

    // Use first path as default
    const path = brand.paths[0] || '/stream1';
    return `rtsp://${cameraIp}:${brand.defaultPort}${path}`;
  };

  const handleAddToConfig = (camera: DiscoveredCamera, cameraIndex: number) => {
    const cameraKey = `camera-${cameraIndex}`;
    const brandIndex = selectedBrand[cameraKey] || 0;
    const brand = CAMERA_BRANDS[brandIndex];
    const rtspUrl = generateRtspUrl(camera.ip, cameraKey);

    // Navigate to cameras page with pre-filled data
    navigate('/cameras', {
      state: {
        prefillCamera: {
          ip: camera.ip,
          rtsp_url: rtspUrl,
          brand: brand.name,
          device_type: camera.device_type,
          ports: camera.ports,
        }
      }
    });
  };

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="bg-white shadow rounded-lg">
        {/* Environment Info */}
        {!isTauri && (
          <div className="px-6 py-4 border-b border-blue-200 bg-blue-50">
            <div className="flex items-start">
              <Wifi className="w-5 h-5 text-blue-600 mt-0.5 mr-3 flex-shrink-0" />
              <div>
                <h3 className="text-sm font-medium text-blue-800">Docker / Web 模式</h3>
                <p className="mt-1 text-sm text-blue-700">
                  您正在使用 Docker/Web 模式。内网摄像头扫描功能已启用，可以正常扫描局域网内的 RTSP 摄像头。
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-semibold text-gray-900 flex items-center gap-2">
                <Camera className="w-7 h-7" />
                摄像头发现
              </h2>
              <p className="mt-1 text-sm text-gray-600">
                扫描内网查找 IP 摄像头设备
              </p>
            </div>
            <button
              onClick={handleQuickScan}
              disabled={isScanning}
              className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isScanning ? (
                <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
              ) : (
                <Play className="w-4 h-4 mr-2" />
              )}
              {isScanning ? '扫描中...' : '快速扫描'}
            </button>
          </div>
        </div>

        {/* Scan Configuration */}
        <div className="px-6 py-4 bg-gray-50">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">扫描配置</h3>
            {lastScanTime && (
              <div className="flex items-center gap-4">
                <span className="text-sm text-gray-500">
                  上次扫描: {new Date(lastScanTime).toLocaleString('zh-CN')}
                </span>
                <button
                  onClick={clearScanResults}
                  className="inline-flex items-center px-3 py-1 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  清除结果
                </button>
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Network Interface Selector (for Tauri) */}
            {isTauri && networkInterfaces.length > 0 && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  网络接口
                </label>
                <select
                  value={selectedInterface?.name || ''}
                  onChange={(e) => handleInterfaceChange(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                >
                  {networkInterfaces.map((iface) => (
                    <option key={iface.name} value={iface.name}>
                      {iface.name} - {iface.ip} {iface.is_default && '(默认)'}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-xs text-gray-500">
                  选择要扫描的网络接口
                </p>
              </div>
            )}

            {/* Network Range */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                网络范围 (CIDR)
              </label>
              <input
                type="text"
                value={networkRange}
                onChange={(e) => setNetworkRange(e.target.value)}
                placeholder="192.168.1.0/24"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                例如: 192.168.1.0/24 扫描 192.168.1.1-254
              </p>
            </div>

            {/* Custom Ports */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                扫描端口 (逗号分隔)
              </label>
              <input
                type="text"
                value={customPorts}
                onChange={(e) => setCustomPorts(e.target.value)}
                placeholder="554,80,8000,8080"
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="mt-1 text-xs text-gray-500">
                常用端口: 554 (RTSP), 80/8000/8080 (HTTP)
              </p>
            </div>
          </div>

          <div className="mt-4">
            <button
              onClick={handleCustomScan}
              disabled={isScanning}
              className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Play className="w-4 h-4 mr-2" />
              自定义扫描
            </button>
          </div>
        </div>

        {/* Progress Bar */}
        {isScanning && (
          <div className="px-6 py-4 border-t border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">扫描进度</span>
              <span className="text-sm text-gray-600">{scanProgress}%</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${scanProgress}%` }}
              />
            </div>
          </div>
        )}

        {/* Error Message */}
        {error && (
          <div className="px-6 py-4 border-t border-gray-200">
            <div className="rounded-md bg-red-50 p-4">
              <div className="flex">
                <AlertCircle className="w-5 h-5 text-red-400" />
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-red-800">扫描错误</h3>
                  <p className="mt-1 text-sm text-red-700">{error}</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Results */}
        {cameras.length > 0 && (
          <div className="px-6 py-4 border-t border-gray-200">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-gray-900">
                发现的设备 ({cameras.length})
              </h3>
            </div>

            <div className="space-y-4">
              {cameras.map((camera, index) => (
                <div
                  key={index}
                  className="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start gap-4">
                      {getDeviceIcon(camera.device_type)}
                      <div>
                        <h4 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
                          {camera.ip}
                          <Check className="w-5 h-5 text-green-500" />
                        </h4>
                        <p className="text-sm text-gray-600 mt-1">
                          {camera.device_type}
                        </p>
                        {camera.hostname && (
                          <p className="text-sm text-gray-500 mt-1">
                            主机名: {camera.hostname}
                          </p>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Open Ports */}
                  <div className="mt-3">
                    <span className="text-sm font-medium text-gray-700">开放端口: </span>
                    <span className="text-sm text-gray-600">
                      {camera.ports.join(', ')}
                    </span>
                  </div>

                  {/* RTSP URLs */}
                  {camera.rtsp_urls.length > 0 && (
                    <div className="mt-3">
                      <p className="text-sm font-medium text-gray-700 mb-2">可能的 RTSP 地址:</p>
                      <div className="space-y-1">
                        {camera.rtsp_urls.slice(0, 3).map((url, idx) => (
                          <div key={idx} className="flex items-center gap-2">
                            <code className="flex-1 text-xs bg-gray-800 text-gray-100 px-2 py-1 rounded font-mono">
                              {url}
                            </code>
                            <button
                              onClick={() => copyToClipboard(url)}
                              className="text-blue-600 hover:text-blue-800 text-xs"
                              title="复制"
                            >
                              复制
                            </button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* HTTP URLs */}
                  {camera.http_urls.length > 0 && (
                    <div className="mt-3">
                      <p className="text-sm font-medium text-gray-700 mb-2">Web 管理地址:</p>
                      {camera.http_urls.map((url, idx) => (
                        <a
                          key={idx}
                          href={url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-flex items-center gap-1 text-sm text-blue-600 hover:text-blue-800"
                        >
                          {url}
                          <ExternalLink className="w-3 h-3" />
                        </a>
                      ))}
                    </div>
                  )}

                  {/* Add to Config Button with Brand Selection */}
                  <div className="mt-4 pt-4 border-t border-gray-200">
                    <div className="space-y-3">
                      {/* Brand Selector */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          摄像头品牌
                        </label>
                        <select
                          value={selectedBrand[`camera-${index}`] || 0}
                          onChange={(e) => setSelectedBrand({
                            ...selectedBrand,
                            [`camera-${index}`]: parseInt(e.target.value)
                          })}
                          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 text-sm"
                        >
                          {CAMERA_BRANDS.map((brand, idx) => (
                            <option key={idx} value={idx}>
                              {brand.name}
                            </option>
                          ))}
                        </select>
                      </div>

                      {/* Show RTSP paths for selected brand */}
                      {CAMERA_BRANDS[selectedBrand[`camera-${index}`] || 0].paths.length > 0 && (
                        <div className="bg-blue-50 rounded-md p-3">
                          <p className="text-xs font-medium text-blue-900 mb-1">
                            该品牌常用 RTSP 路径：
                          </p>
                          <div className="space-y-1">
                            {CAMERA_BRANDS[selectedBrand[`camera-${index}`] || 0].paths.map((path, pidx) => (
                              <code key={pidx} className="block text-xs text-blue-700 font-mono">
                                rtsp://{camera.ip}:{CAMERA_BRANDS[selectedBrand[`camera-${index}`] || 0].defaultPort}{path}
                              </code>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Custom path input for custom brand */}
                      {CAMERA_BRANDS[selectedBrand[`camera-${index}`] || 0].name === '自定义' && (
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-1">
                            自定义 RTSP 路径
                          </label>
                          <input
                            type="text"
                            value={customPath[`camera-${index}`] || '/'}
                            onChange={(e) => setCustomPath({
                              ...customPath,
                              [`camera-${index}`]: e.target.value
                            })}
                            placeholder="/stream1"
                            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 text-sm font-mono"
                          />
                          <p className="mt-1 text-xs text-gray-500">
                            例如: /stream1, /h264/ch1/main/av_stream
                          </p>
                        </div>
                      )}

                      {/* Add to Config Button */}
                      <button
                        className="w-full inline-flex items-center justify-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        onClick={() => handleAddToConfig(camera, index)}
                      >
                        <Camera className="w-4 h-4 mr-2" />
                        添加到配置
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Empty State */}
        {!isScanning && cameras.length === 0 && !error && (
          <div className="px-6 py-12 text-center">
            <Wifi className="mx-auto w-12 h-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">
              尚未扫描
            </h3>
            <p className="mt-1 text-sm text-gray-500">
              点击"快速扫描"开始查找内网摄像头
            </p>
          </div>
        )}
      </div>

      {/* Help Section */}
      <div className="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="text-sm font-medium text-blue-900 mb-2">💡 使用提示</h4>
        <ul className="text-sm text-blue-800 space-y-1 list-disc list-inside">
          <li>快速扫描会自动检测您的网络并扫描常用端口 (554, 80, 8000, 8080)</li>
          <li>自定义扫描允许您指定网络范围和端口列表</li>
          <li>扫描结果会自动保存，页面刷新后仍然保留（24小时内有效）</li>
          <li>可以选择不同的网络接口进行扫描（如有多个网卡）</li>
          <li>扫描可能需要 1-2 分钟,具体取决于网络大小</li>
          <li>确保摄像头和本机在同一网络内</li>
          <li>某些摄像头可能需要登录才能访问 RTSP 流</li>
        </ul>
      </div>
    </div>
  );
};

export default CameraDiscoveryPage;
