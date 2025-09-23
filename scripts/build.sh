#!/bin/bash

# C-drive空间管理器 构建脚本
# 支持多平台构建和部署

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 默认配置
PLATFORM="all"
ARCH="x86_64"
MODE="release"
VERSION=$(node -p "require('./package.json').version")
APP_NAME="C-drive空间管理器"

# 帮助信息
show_help() {
    cat << EOF
C-drive空间管理器 构建脚本

用法: $0 [选项]

选项:
    -p, --platform <platform>   目标平台 (windows, linux, macos, all) [默认: all]
    -a, --arch <architecture>   架构 (x86_64, aarch64) [默认: x86_64]
    -m, --mode <mode>           构建模式 (debug, release) [默认: release]
    -v, --version <version>     版本号 [默认: package.json中的版本]
    -c, --clean                 清理构建缓存
    -t, --test                  运行测试
    -s, --sign                  代码签名 (仅Windows)
    -d, --deploy                部署到发布服务器
    -h, --help                  显示帮助信息

示例:
    $0                          # 构建所有平台的发布版本
    $0 -p windows -s           # 构建Windows版本并签名
    $0 -p linux -m debug       # 构建Linux调试版本
    $0 -c -t                   # 清理缓存并运行测试

EOF
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -t|--test)
            TEST=true
            shift
            ;;
        -s|--sign)
            SIGN=true
            shift
            ;;
        -d|--deploy)
            DEPLOY=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 检查依赖
check_dependencies() {
    log_info "检查构建依赖..."
    
    # 检查Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js 未安装"
        exit 1
    fi
    
    # 检查Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust 未安装"
        exit 1
    fi
    
    # 检查pnpm
    if ! command -v pnpm &> /dev/null; then
        log_error "pnpm 未安装"
        exit 1
    fi
    
    # 检查Tauri CLI
    if ! command -v tauri &> /dev/null; then
        log_error "Tauri CLI 未安装"
        exit 1
    fi
    
    log_success "所有依赖检查通过"
}

# 清理构建缓存
clean_build() {
    if [ "$CLEAN" = true ]; then
        log_info "清理构建缓存..."
        
        # 清理前端构建缓存
        rm -rf dist
        rm -rf node_modules/.vite
        rm -rf node_modules/.cache
        
        # 清理Rust构建缓存
        cd src-tauri
        cargo clean
        cd ..
        
        # 清理Tauri缓存
        rm -rf src-tauri/target
        
        log_success "构建缓存清理完成"
    fi
}

# 安装依赖
install_dependencies() {
    log_info "安装项目依赖..."
    
    # 安装前端依赖
    pnpm install --frozen-lockfile
    
    # 安装Rust依赖
    cd src-tauri
    cargo fetch
    cd ..
    
    log_success "依赖安装完成"
}

# 运行测试
run_tests() {
    if [ "$TEST" = true ]; then
        log_info "运行测试套件..."
        
        # 前端测试
        pnpm run test
        
        # Rust测试
        cd src-tauri
        cargo test
        cd ..
        
        # 集成测试
        pnpm run test:integration
        
        log_success "测试运行完成"
    fi
}

# 构建前端
build_frontend() {
    log_info "构建前端应用..."
    
    # 使用生产环境配置
    export NODE_ENV=production
    
    if [ "$MODE" = "release" ]; then
        pnpm run build:prod
    else
        pnpm run build
    fi
    
    log_success "前端构建完成"
}

# 构建Rust后端
build_backend() {
    log_info "构建Rust后端..."
    
    cd src-tauri
    
    if [ "$MODE" = "release" ]; then
        # 使用生产环境配置
        cp Cargo.prod.toml Cargo.toml
        cp tauri.conf.prod.json tauri.conf.json
        
        # 发布构建
        cargo build --release
    else
        # 调试构建
        cargo build
    fi
    
    cd ..
    
    log_success "Rust后端构建完成"
}

# 代码签名 (仅Windows)
sign_code() {
    if [ "$SIGN" = true ] && [ "$PLATFORM" = "windows" ]; then
        log_info "进行代码签名..."
        
        # 检查签名证书
        if [ -z "$CODE_SIGN_CERT" ]; then
            log_warning "未设置代码签名证书，跳过签名"
            return
        fi
        
        # 签名可执行文件
        cd src-tauri/target/release/bundle/msi
        signtool sign /f "$CODE_SIGN_CERT" /p "$CODE_SIGN_PASSWORD" /t http://timestamp.digicert.com /v *.msi
        cd ../../../../..
        
        log_success "代码签名完成"
    fi
}

# 打包应用
package_app() {
    log_info "打包应用程序..."
    
    # 使用Tauri打包
    if [ "$PLATFORM" = "all" ]; then
        pnpm tauri build --target universal-apple-darwin
        pnpm tauri build --target x86_64-pc-windows-msvc
        pnpm tauri build --target x86_64-unknown-linux-gnu
    else
        case $PLATFORM in
            windows)
                pnpm tauri build --target x86_64-pc-windows-msvc
                ;;
            linux)
                pnpm tauri build --target x86_64-unknown-linux-gnu
                ;;
            macos)
                pnpm tauri build --target universal-apple-darwin
                ;;
        esac
    fi
    
    log_success "应用打包完成"
}

# 生成校验和
generate_checksums() {
    log_info "生成文件校验和..."
    
    cd src-tauri/target/release/bundle
    
    # 为所有安装包生成校验和
    find . -name "*.msi" -o -name "*.dmg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" | while read file; do
        sha256sum "$file" > "${file}.sha256"
        log_info "生成校验和: ${file}.sha256"
    done
    
    cd ../../../..
    
    log_success "校验和生成完成"
}

# 部署到发布服务器
deploy_to_server() {
    if [ "$DEPLOY" = true ]; then
        log_info "部署到发布服务器..."
        
        # 检查部署配置
        if [ -z "$DEPLOY_SERVER" ] || [ -z "$DEPLOY_PATH" ]; then
            log_error "未设置部署服务器配置"
            exit 1
        fi
        
        # 上传文件
        cd src-tauri/target/release/bundle
        rsync -avz --progress . "$DEPLOY_SERVER:$DEPLOY_PATH/"
        cd ../../../..
        
        log_success "部署完成"
    fi
}

# 生成发布说明
generate_release_notes() {
    log_info "生成发布说明..."
    
    cat > RELEASE_NOTES.md << EOF
# C-drive空间管理器 v${VERSION} 发布说明

## 版本信息
- **版本**: ${VERSION}
- **构建时间**: $(date '+%Y-%m-%d %H:%M:%S')
- **构建平台**: ${PLATFORM}
- **架构**: ${ARCH}
- **构建模式**: ${MODE}

## 系统要求
- **操作系统**: Windows 10/11, macOS 10.13+, Ubuntu 18.04+
- **内存**: 4GB RAM (推荐8GB)
- **磁盘空间**: 100MB可用空间
- **权限**: 管理员权限（推荐）

## 安装包
$(find src-tauri/target/release/bundle -name "*.msi" -o -name "*.dmg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" | sort)

## 校验和
$(find src-tauri/target/release/bundle -name "*.sha256" | xargs cat)

## 更新内容
请查看 CHANGELOG.md 获取详细的更新内容。

## 技术支持
- **文档**: https://docs.dir-mover.com
- **支持邮箱**: support@dir-mover.com
- **社区论坛**: https://forum.dir-mover.com

---
Copyright © 2024 Dir-Mover Technologies. All rights reserved.
EOF
    
    log_success "发布说明生成完成"
}

# 主构建流程
main() {
    log_info "开始构建 C-drive空间管理器 v${VERSION}"
    log_info "目标平台: ${PLATFORM}, 架构: ${ARCH}, 模式: ${MODE}"
    
    # 检查依赖
    check_dependencies
    
    # 清理缓存（如果需要）
    clean_build
    
    # 安装依赖
    install_dependencies
    
    # 运行测试（如果需要）
    run_tests
    
    # 构建前端
    build_frontend
    
    # 构建后端
    build_backend
    
    # 打包应用
    package_app
    
    # 代码签名（如果需要）
    sign_code
    
    # 生成校验和
    generate_checksums
    
    # 生成发布说明
    generate_release_notes
    
    # 部署（如果需要）
    deploy_to_server
    
    log_success "构建完成！"
    log_info "安装包位置: src-tauri/target/release/bundle/"
    log_info "发布说明: RELEASE_NOTES.md"
}

# 运行主函数
main "$@"