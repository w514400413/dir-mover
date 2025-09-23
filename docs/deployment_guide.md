# C-drive空间管理器 部署指南

## 目录
1. [概述](#概述)
2. [系统要求](#系统要求)
3. [构建环境准备](#构建环境准备)
4. [本地构建](#本地构建)
5. [CI/CD部署](#cicd部署)
6. [Docker部署](#docker部署)
7. [多平台发布](#多平台发布)
8. [自动化部署](#自动化部署)
9. [监控和维护](#监控和维护)
10. [故障排除](#故障排除)

## 概述

本文档详细介绍了C-drive空间管理器的部署流程，包括本地构建、CI/CD自动化部署、Docker容器化部署以及多平台发布策略。

## 系统要求

### 构建环境要求
- **操作系统**: Windows 10/11, macOS 10.15+, Ubuntu 20.04+
- **Node.js**: 18.0.0 或更高版本
- **Rust**: 1.75.0 或更高版本
- **pnpm**: 8.0.0 或更高版本
- **Tauri CLI**: 2.0.0 或更高版本
- **Docker**: 20.10+ (可选，用于容器化构建)
- **内存**: 8GB RAM (推荐16GB)
- **磁盘空间**: 10GB 可用空间

### 目标平台要求
| 平台 | 最低版本 | 架构支持 | 特殊要求 |
|------|----------|----------|----------|
| Windows | 10 (1903+) | x86_64 | WebView2 运行时 |
| macOS | 10.13 | x86_64, ARM64 | 无 |
| Linux | Ubuntu 18.04 | x86_64 | WebKit2GTK |

## 构建环境准备

### 1. 安装Node.js
```bash
# 使用Node版本管理器 (推荐)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# 或者直接从官网下载安装
# https://nodejs.org/
```

### 2. 安装Rust
```bash
# 官方安装脚本
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 安装pnpm
```bash
# 使用npm安装
npm install -g pnpm

# 验证安装
pnpm --version
```

### 4. 安装Tauri CLI
```bash
# 安装Tauri CLI
cargo install tauri-cli

# 验证安装
tauri --version
```

### 5. 平台特定依赖

#### Windows
```powershell
# 安装WebView2运行时
# 下载链接: https://developer.microsoft.com/microsoft-edge/webview2/
# 安装Visual Studio Build Tools (可选)
```

#### macOS
```bash
# 安装Xcode命令行工具
xcode-select --install

# 安装Homebrew (可选)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Linux (Ubuntu/Debian)
```bash
# 安装系统依赖
sudo apt update
sudo apt install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf

# 安装构建工具
sudo apt install -y build-essential curl wget file libssl-dev
```

## 本地构建

### 快速构建

#### Windows
```batch
# 使用构建脚本
scripts\build.bat

# 或者手动构建
pnpm install
pnpm build
pnpm tauri build
```

#### macOS/Linux
```bash
# 使用构建脚本
chmod +x scripts/build.sh
./scripts/build.sh

# 或者手动构建
pnpm install
pnpm build
pnpm tauri build
```

### 高级构建选项

#### 生产环境构建
```bash
# 使用生产配置
pnpm run build:prod
pnpm run tauri:build:prod
```

#### 多平台构建
```bash
# 构建所有平台
./scripts/build.sh -p all -m release

# 构建特定平台
./scripts/build.sh -p windows -m release
./scripts/build.sh -p linux -m release
./scripts/build.sh -p macos -m release
```

#### 带代码签名的构建
```bash
# Windows代码签名
set CODE_SIGN_CERT=path/to/certificate.pfx
set CODE_SIGN_PASSWORD=your_password
./scripts/build.bat -p windows -s

# macOS代码签名
export APPLE_CERTIFICATE=path/to/certificate.p12
export APPLE_CERTIFICATE_PASSWORD=your_password
./scripts/build.sh -p macos -s
```

## CI/CD部署

### GitHub Actions配置

项目已配置完整的GitHub Actions工作流：

#### 触发条件
- 推送版本标签 (`v*`)
- 手动触发工作流
- 定时构建 (可选)

#### 构建流程
1. **测试阶段**: 运行单元测试和集成测试
2. **前端构建**: 构建和优化前端资源
3. **多平台构建**: 并行构建Windows、Linux、macOS版本
4. **代码签名**: 对发布版本进行数字签名
5. **创建发布**: 自动生成GitHub Release
6. **部署**: 部署到发布服务器

#### 使用方式
```yaml
# 手动触发构建
# 进入GitHub仓库 -> Actions -> Release Build -> Run workflow

# 自动触发
# 推送版本标签
git tag v1.0.0
git push origin v1.0.0
```

### 环境变量配置

在GitHub仓库设置中添加以下Secrets：

#### 代码签名
```bash
# Windows
CODE_SIGN_CERT          # Base64编码的证书文件
CODE_SIGN_PASSWORD      # 证书密码

# macOS
APPLE_CERTIFICATE       # Base64编码的开发者证书
APPLE_CERTIFICATE_PASSWORD  # 证书密码
APPLE_SIGNING_IDENTITY  # 签名身份ID
```

#### 部署配置
```bash
DEPLOY_SERVER          # 部署服务器地址
DEPLOY_PATH           # 部署路径
DEPLOY_KEY            # SSH部署密钥
```

## Docker部署

### 使用Docker Compose

#### 开发环境
```bash
# 启动开发环境
docker-compose --profile dev up -d

# 访问应用
# 前端开发服务器: http://localhost:1420
```

#### 构建环境
```bash
# 启动构建环境
docker-compose --profile build up -d

# 执行构建
docker-compose exec builder ./scripts/build.sh
```

#### 测试环境
```bash
# 启动测试环境
docker-compose --profile test up -d

# 运行测试
docker-compose exec test pnpm test
```

### 自定义Docker构建

#### 多阶段构建
```dockerfile
# 使用项目提供的Dockerfile
docker build -t c-drive-space-manager-builder .

# 运行构建容器
docker run -v $(pwd)/output:/app/output c-drive-space-manager-builder ./scripts/build.sh
```

#### 优化构建缓存
```bash
# 构建并缓存依赖层
docker build --target frontend-builder -t c-drive-frontend .
docker build --target rust-builder -t c-drive-rust .

# 最终构建
docker build -t c-drive-space-manager .
```

## 多平台发布

### 发布策略

#### 1. 版本管理
- 使用语义化版本控制 (SemVer)
- 主版本号: 重大功能更新
- 次版本号: 新功能添加
- 修订号: Bug修复和优化

#### 2. 发布渠道
- **稳定版**: 经过充分测试的生产版本
- **测试版**: 新功能预览版本
- **开发版**: 日常构建版本

#### 3. 平台支持
| 平台 | 安装包格式 | 发布方式 |
|------|------------|----------|
| Windows | MSI, NSIS | GitHub Release |
| macOS | DMG | GitHub Release |
| Linux | DEB, RPM, AppImage | GitHub Release |

### 自动更新机制

#### 更新检查
```rust
// Tauri配置中的更新设置
"updater": {
  "active": true,
  "endpoints": [
    "https://api.dir-mover.com/releases/{{target}}/{{arch}}/{{current_version}}"
  ],
  "dialog": true,
  "pubkey": "YOUR_PUBLIC_KEY"
}
```

#### 更新服务器
```bash
# 简单的更新服务器配置
# 使用Nginx提供静态文件服务
server {
    listen 80;
    server_name api.dir-mover.com;
    
    location /releases/ {
        alias /var/www/releases/;
        autoindex on;
    }
}
```

## 自动化部署

### 部署脚本

#### 完整部署流程
```bash
#!/bin/bash
# deploy.sh - 完整部署脚本

set -e

# 配置变量
VERSION=$1
PLATFORM=$2
DEPLOY_SERVER=$3

# 验证参数
if [ -z "$VERSION" ] || [ -z "$PLATFORM" ] || [ -z "$DEPLOY_SERVER" ]; then
    echo "用法: $0 <version> <platform> <deploy_server>"
    exit 1
fi

# 构建应用
echo "开始构建版本 $VERSION..."
./scripts/build.sh -p $PLATFORM -v $VERSION -m release

# 运行测试
echo "运行测试..."
./scripts/build.sh -p $PLATFORM -t

# 生成校验和
echo "生成校验和..."
cd src-tauri/target/release/bundle
find . -name "*.msi" -o -name "*.dmg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" | xargs -I {} sh -c 'sha256sum {} > {}.sha256'

# 部署到服务器
echo "部署到服务器 $DEPLOY_SERVER..."
rsync -avz --progress . "$DEPLOY_SERVER:/var/www/releases/$VERSION/"

# 更新最新版本链接
ssh "$DEPLOY_SERVER" "ln -sfn /var/www/releases/$VERSION /var/www/releases/latest"

echo "部署完成！"
```

### 蓝绿部署

#### 零停机部署策略
```bash
# 蓝绿部署脚本
#!/bin/bash

# 当前活跃环境
CURRENT_ENV=$(ssh deploy-server "readlink /var/www/active")

# 目标环境
if [ "$CURRENT_ENV" = "blue" ]; then
    TARGET_ENV="green"
else
    TARGET_ENV="blue"
fi

# 部署到新环境
echo "部署到 $TARGET_ENV 环境..."
rsync -avz --delete build/ "deploy-server:/var/www/$TARGET_ENV/"

# 健康检查
echo "进行健康检查..."
if curl -f http://deploy-server:8080/health; then
    # 切换流量
    echo "切换流量到 $TARGET_ENV..."
    ssh deploy-server "ln -sfn /var/www/$TARGET_ENV /var/www/active"
    
    # 清理旧环境
    echo "清理旧环境..."
    # 保留旧版本用于回滚
else
    echo "健康检查失败，回滚..."
    exit 1
fi
```

## 监控和维护

### 应用监控

#### 性能监控
```javascript
// 前端性能监控
import { performance } from 'perf_hooks';

// 监控页面加载时间
window.addEventListener('load', () => {
    const loadTime = performance.timing.loadEventEnd - performance.timing.navigationStart;
    console.log(`页面加载时间: ${loadTime}ms`);
    
    // 发送到监控服务
    fetch('/api/metrics/performance', {
        method: 'POST',
        body: JSON.stringify({ loadTime, timestamp: Date.now() })
    });
});
```

#### 错误监控
```rust
// Rust后端错误监控
use log::{error, info};

fn log_error(error: &Error) {
    error!("应用错误: {:?}", error);
    
    // 发送到错误监控服务
    if let Ok(client) = reqwest::Client::new() {
        let _ = client.post("https://monitor.dir-mover.com/errors")
            .json(&json!({
                "error": error.to_string(),
                "timestamp": chrono::Utc::now(),
                "version": env!("CARGO_PKG_VERSION")
            }))
            .send();
    }
}
```

### 日志管理

#### 日志配置
```rust
// 日志配置
use simplelog::*;

fn setup_logging() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            std::fs::File::create("logs/app.log")?,
        ),
    ])?;
    Ok(())
}
```

#### 日志轮转
```bash
# 使用logrotate进行日志轮转
# /etc/logrotate.d/c-drive-space-manager
/var/log/c-drive-space-manager/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 app app
    postrotate
        # 重启应用以重新打开日志文件
        systemctl reload c-drive-space-manager
    endscript
}
```

## 故障排除

### 常见构建问题

#### 1. 依赖安装失败
```bash
# 清理缓存
pnpm store prune
rm -rf node_modules
pnpm install --frozen-lockfile

# Rust依赖
cd src-tauri
cargo clean
cargo fetch
```

#### 2. 构建内存不足
```bash
# 增加Node.js内存限制
export NODE_OPTIONS="--max-old-space-size=4096"

# Rust编译优化
export CARGO_BUILD_JOBS=2
```

#### 3. 平台特定问题

##### Windows
```powershell
# WebView2安装问题
# 手动下载安装: https://developer.microsoft.com/microsoft-edge/webview2/

# 路径长度限制
# 启用长路径支持: gpedit.msc -> 计算机配置 -> 管理模板 -> 系统 -> 文件系统
```

##### macOS
```bash
# Xcode许可问题
sudo xcodebuild -license accept

# 签名问题
security find-identity -v -p codesigning
```

##### Linux
```bash
# 依赖库问题
sudo apt install -y libwebkit2gtk-4.0-dev libgtk-3-dev

# 权限问题
sudo usermod -a -G sudo $USER
```

### 部署问题排查

#### 1. 构建失败
```bash
# 查看详细日志
RUST_BACKTRACE=1 cargo build --verbose

# 检查依赖版本
cargo tree
pnpm list
```

#### 2. 运行时错误
```bash
# 查看应用日志
tail -f logs/app.log

# 系统日志
journalctl -u c-drive-space-manager -f
```

#### 3. 性能问题
```bash
# 监控系统资源
htop
iotop

# 分析构建性能
cargo build --timings
```

## 最佳实践

### 1. 版本控制
- 使用Git标签管理版本
- 维护详细的CHANGELOG
- 遵循语义化版本规范

### 2. 构建优化
- 使用构建缓存
- 并行构建多个平台
- 优化依赖项

### 3. 安全实践
- 代码签名所有发布版本
- 使用安全的部署通道
- 定期更新依赖项

### 4. 监控告警
- 设置构建失败告警
- 监控发布服务器状态
- 跟踪用户反馈

---

**部署指南版本**: v1.0  
**最后更新**: 2024年1月  
**适用范围**: C-drive空间管理器 v1.0.0及以上版本

如需更多帮助，请联系技术支持：support@dir-mover.com