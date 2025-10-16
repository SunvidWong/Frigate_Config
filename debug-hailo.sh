#!/bin/bash
# Hailo 硬件检测调试脚本

echo "=========================================="
echo "Hailo 硬件检测调试脚本"
echo "=========================================="
echo ""

# 1. 检查 Hailo 设备文件
echo "1. 检查 /dev/hailo* 设备："
ls -la /dev/hailo* 2>/dev/null
if [ $? -ne 0 ]; then
    echo "   未找到 /dev/hailo* 设备"
else
    echo "   找到 Hailo 设备文件"
fi
echo ""

# 2. 检查 Hailo 内核模块
echo "2. 检查 Hailo 内核模块："
lsmod | grep -i hailo
if [ $? -ne 0 ]; then
    echo "   未找到 hailo 内核模块"
    echo "   尝试加载模块："
    echo "   sudo modprobe hailo_pci"
else
    echo "   Hailo 模块已加载"
fi
echo ""

# 3. 检查 PCIe 设备 (Hailo vendor ID: 0x1e60)
echo "3. 检查 PCIe 设备 (Hailo vendor ID: 0x1e60)："
lspci -nn | grep -i "1e60"
if [ $? -ne 0 ]; then
    echo "   未通过 lspci 找到 Hailo PCIe 设备"
    echo "   尝试查找所有 AI 加速器："
    lspci -nn | grep -i -E "accelerator|processing unit|ai"
else
    echo "   找到 Hailo PCIe 设备"
fi
echo ""

# 4. 检查 sysfs 中的 Hailo 设备
echo "4. 检查 sysfs 中的 vendor ID 0x1e60："
for device in /sys/bus/pci/devices/*; do
    if [ -f "$device/vendor" ]; then
        vendor=$(cat "$device/vendor" 2>/dev/null)
        if [ "$vendor" = "0x1e60" ]; then
            echo "   找到 Hailo 设备: $device"
            echo "   Vendor ID: $vendor"
            if [ -f "$device/device" ]; then
                echo "   Device ID: $(cat $device/device)"
            fi
            if [ -f "$device/class" ]; then
                echo "   Class: $(cat $device/class)"
            fi
        fi
    fi
done
echo ""

# 5. 检查 dmesg 中的 Hailo 相关消息
echo "5. 检查 dmesg 中的 Hailo 消息："
sudo dmesg | grep -i hailo | tail -5
echo ""

# 6. 检查 Hailo 软件工具
echo "6. 检查 Hailo 软件工具："
which hailo 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   找到 hailo 命令"
    hailo --version 2>/dev/null
else
    echo "   未找到 hailo 命令行工具"
fi

which hailortcli 2>/dev/null
if [ $? -eq 0 ]; then
    echo "   找到 hailortcli 命令"
    hailortcli --version 2>/dev/null
else
    echo "   未找到 hailortcli 命令"
fi
echo ""

# 7. 运行 Agent 硬件检测
echo "7. 运行 Agent 硬件检测："
if [ -f "./agent/agent" ]; then
    echo "   运行 ./agent/agent detect --json"
    ./agent/agent detect --json 2>/dev/null | jq '.tpus[] | select(.name | contains("Hailo"))'
else
    echo "   Agent 未找到，请先编译: cd agent && go build"
fi
echo ""

# 8. 检查权限
echo "8. 检查设备权限："
if [ -e "/dev/hailo0" ]; then
    ls -la /dev/hailo0
    echo "   当前用户: $(whoami)"
    echo "   用户组: $(groups)"
    echo ""
    echo "   如果权限不足，尝试："
    echo "   sudo chmod 666 /dev/hailo0"
    echo "   或将用户添加到相应组"
fi
echo ""

echo "=========================================="
echo "调试完成"
echo "=========================================="
echo ""
echo "常见问题解决方案："
echo "1. 如果没有 /dev/hailo* 设备："
echo "   - 确认 Hailo 驱动已安装"
echo "   - 运行: sudo modprobe hailo_pci"
echo ""
echo "2. 如果 lspci 看不到设备："
echo "   - 确认 Hailo 卡已正确安装在 PCIe 插槽"
echo "   - 检查 BIOS 设置中的 PCIe 配置"
echo ""
echo "3. 如果权限不足："
echo "   - 运行: sudo chmod 666 /dev/hailo*"
echo "   - 或将用户添加到 video 组: sudo usermod -a -G video $(whoami)"
echo ""