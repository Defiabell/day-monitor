#!/usr/bin/env bash
# Day Monitor 一键安装脚本
# 用法：把 install.sh 和 Day-Monitor-0.1.0-arm64.zip 放同一目录，运行 ./install.sh

set -euo pipefail

ZIP="Day-Monitor-0.1.0-universal.zip"
APP_NAME="Day Monitor.app"
DEST="/Applications"

cd "$(dirname "$0")"

if [[ ! -f "$ZIP" ]]; then
    echo "❌ 找不到 $ZIP（应该和这个脚本放同一目录）"
    exit 1
fi

# 1. 解压
echo "📦 解压 $ZIP ..."
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT
ditto -x -k "$ZIP" "$TMP"

if [[ ! -d "$TMP/$APP_NAME" ]]; then
    echo "❌ 解压后没找到 $APP_NAME"
    exit 1
fi

# 2. 移到 /Applications
echo "📁 安装到 $DEST ..."
if [[ -d "$DEST/$APP_NAME" ]]; then
    echo "   覆盖已有版本"
    rm -rf "$DEST/$APP_NAME"
fi
cp -R "$TMP/$APP_NAME" "$DEST/"

# 3. 解除 Gatekeeper 隔离
echo "🔓 解除 Gatekeeper 隔离 ..."
xattr -dr com.apple.quarantine "$DEST/$APP_NAME" 2>/dev/null || true

# 3b. 清旧 TCC 记录（如果是升级安装，旧版本的截屏授权对不上新 binary 的 hash，必须清掉）
echo "🧹 清除旧版本的截屏授权记录（如有）..."
tccutil reset ScreenCapture com.jinkunsun.daymonitor 2>/dev/null || true
tccutil reset All com.jinkunsun.daymonitor 2>/dev/null || true

# 4. 提示 API Key
ENV_FILE="$HOME/.day-monitor/.env"
if [[ ! -f "$ENV_FILE" ]]; then
    echo ""
    echo "🔑 还需要配置 Anthropic API Key（去 https://console.anthropic.com 注册一个）"
    read -r -p "   现在输入 API Key（粘贴后回车，留空跳过稍后从 Settings 输入）: " KEY
    if [[ -n "$KEY" ]]; then
        mkdir -p "$HOME/.day-monitor"
        echo "ANTHROPIC_API_KEY=$KEY" > "$ENV_FILE"
        chmod 600 "$ENV_FILE"
        echo "   ✓ 写入 $ENV_FILE"
    fi
fi

# 5. 启动
echo ""
echo "🚀 启动 Day Monitor ..."
open -a "Day Monitor"

echo ""
echo "✅ 安装完成！"
echo ""
echo "   - 菜单栏右上角会出现一个图标"
echo "   - 第一次启动会弹截屏权限请求 → 「打开系统设置」 → 在「录屏与系统录音」里打开 Day Monitor 开关"
echo "   - 授权后退出 app（菜单栏右键 → Quit）再重新打开"
echo ""
echo "📚 更多说明：见 INSTALL.md"
