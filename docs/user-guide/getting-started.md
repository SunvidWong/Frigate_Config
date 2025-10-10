# Getting Started Guide

Welcome to the Frigate Configuration Tool! This guide will help you set up and deploy Frigate NVR in minutes.

## Prerequisites

Before you begin, ensure you have:

- **Docker installed and running**
  - Linux: `sudo systemctl status docker`
  - macOS: Docker Desktop running
  - Windows: Docker Desktop running
- **Sufficient disk space** (minimum 50GB recommended for recordings)
- **Camera RTSP URLs** (for your IP cameras)

## Installation

### Linux

1. Download the AppImage:
   ```bash
   curl -LO https://github.com/YOUR_ORG/frigate-config/releases/latest/download/frigate-config-tool.AppImage
   ```

2. Make it executable:
   ```bash
   chmod +x frigate-config-tool.AppImage
   ```

3. Run:
   ```bash
   ./frigate-config-tool.AppImage
   ```

**Alternative: Debian/Ubuntu Package**
```bash
sudo dpkg -i frigate-config-tool_*.deb
frigate-config-tool
```

### macOS

1. Download the DMG file
2. Open the DMG
3. Drag "Frigate Config Tool" to Applications
4. First launch: Right-click → Open (bypass Gatekeeper)

### Windows

1. Download the MSI installer
2. Double-click to install
3. Launch from Start Menu

## Quick Setup (5 Minutes)

### Step 1: Hardware Detection

1. Launch the application
2. Click **Hardware** in the sidebar
3. Click **Detect Hardware** button
4. Wait for detection to complete (5-10 seconds)

**Expected Results**:
- Intel QuickSync devices (if available)
- NVIDIA GPUs (if available)
- AMD GPUs (if available)
- Coral TPU (if connected)

### Step 2: Configure Cameras

1. Click **Cameras** in the sidebar
2. Click **+ Add Camera**
3. Fill in camera details:
   - **Name**: `front_door` (use underscores, not spaces)
   - **RTSP URL**: `rtsp://username:password@camera-ip:554/stream`
   - **Resolution**: Select from dropdown (e.g., 1920x1080)
   - **FPS**: 5-15 recommended
4. Select hardware accelerator (from Step 1)
5. Click **Save**

**Repeat** for each camera.

### Step 3: Configure Storage

1. Click **Disk Mapping** in the sidebar
2. Click **Scan Recommended Paths** (or browse manually)
3. Select paths for:
   - **Recordings** (needs most space: 100GB+)
   - **Clips** (moderate space: 20GB+)
   - **Cache** (small space: 10GB+)
   - **Config** (minimal space: <1GB)
4. Review disk space warnings
5. Click **Save Mappings**

### Step 4: Deploy Frigate

1. Click **Deploy** in the sidebar
2. Review configuration preview
3. Click **Validate Configuration**
4. Wait for validation checks (Docker, ports, paths)
5. If all checks pass, click **Deploy**
6. Monitor deployment progress:
   - Container creation
   - Health checks
   - Frigate API availability

**Success!** Frigate is now running at http://localhost:5000

## Next Steps

### Access Frigate Web UI

Open your browser and navigate to:
```
http://localhost:5000
```

You should see the Frigate dashboard with your cameras.

### View Logs

1. Click **Logs** in the sidebar
2. Monitor real-time container logs
3. Use search/filter to find specific events
4. Export logs if needed

### Manage Configuration

#### Manual Editing

1. Click **Manual Config** in the sidebar
2. Edit YAML directly with syntax highlighting
3. Save changes (automatic backup created)

#### Rollback Changes

1. Go to **Manual Config**
2. Click **Backups** dropdown
3. Select a backup to restore
4. Confirm rollback

## Common Tasks

### Add a New Camera

1. **Cameras** → **+ Add Camera**
2. Configure camera settings
3. Assign hardware accelerator
4. Save and redeploy

### Change Storage Location

1. **Disk Mapping** → Select new path
2. **Deploy** → Redeploy with new volumes
3. Old data remains in previous location

### Update Frigate Image

1. **Deploy** → **Container Options**
2. Change image tag (e.g., `latest` → `0.13.0`)
3. Click **Deploy** (creates new container)
4. Old container automatically stopped

### Troubleshooting Deployment

1. **Deploy** → **Validate Configuration**
2. Review any errors or warnings
3. Fix issues (ports, paths, permissions)
4. Try deployment again

## Tips & Best Practices

### Camera Configuration

- **Use lower FPS** (5-10) to save resources
- **Enable hardware acceleration** for better performance
- **Test RTSP URL** in VLC before adding to Frigate
- **Use consistent naming** (e.g., `location_camera`)

### Storage Planning

- **Recordings**: 1-2GB per camera per day
- **Clips**: Depends on detection frequency
- **Cache**: 2x RAM recommended
- **Separate drives** for recordings if possible

### Performance

- **Intel QuickSync**: Best for multiple cameras
- **NVIDIA**: Excellent for AI detection
- **Coral TPU**: Best object detection
- **CPU only**: Works but limited cameras

### Security

- **Change default ports** if exposed to internet
- **Use strong camera passwords**
- **Keep Frigate updated**
- **Backup configuration regularly**

## Advanced Features

### Docker Compose Mode

1. **Deploy** → **Deployment Method** → Docker Compose
2. Review generated `docker-compose.yml`
3. Deploy using Compose

### Environment Variables

1. **Deploy** → **Environment Variables**
2. Add custom variables (e.g., `TZ=America/New_York`)
3. Save and deploy

### Volume Permissions

If you encounter permission errors:

**Linux**:
```bash
sudo chown -R $(id -u):$(id -g) /path/to/frigate
```

**macOS**:
No action needed (Docker handles permissions)

**Windows**:
Ensure Docker has access to the drive in Docker Desktop settings

## Getting Help

### Check Logs

1. **Logs** page shows container output
2. Export logs for sharing
3. Check for error messages

### Validation Errors

- **Port in use**: Change Frigate port or stop conflicting service
- **Path not found**: Ensure directory exists
- **Docker not running**: Start Docker daemon
- **Permission denied**: Fix directory permissions

### Community Support

- **GitHub Issues**: https://github.com/YOUR_ORG/frigate-config/issues
- **Discussions**: https://github.com/YOUR_ORG/frigate-config/discussions
- **Frigate Docs**: https://docs.frigate.video

## Keyboard Shortcuts

- `Ctrl/Cmd + S`: Save configuration
- `Ctrl/Cmd + K`: Open command palette
- `Ctrl/Cmd + ,`: Open settings
- `Ctrl/Cmd + R`: Reload hardware detection

## What's Next?

- [Configuration Guide](./configuration.md) - Detailed configuration options
- [Deployment Guide](./deployment.md) - Advanced deployment scenarios
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
- [API Reference](../api/README.md) - For developers

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
