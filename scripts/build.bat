@echo off
setlocal enabledelayedexpansion

:: C-drive空间管理器 Windows构建脚本
:: 支持Windows平台的构建和部署

:: 设置控制台编码
chcp 65001 >nul 2>&1

:: 颜色定义
set "RED=[31m"
set "GREEN=[32m"
set "YELLOW=[33m"
set "BLUE=[34m"
set "NC=[0m"

:: 日志函数
:log_info
echo %BLUE%[INFO]%NC% %~1
goto :eof

:log_success
echo %GREEN%[SUCCESS]%NC% %~1
goto :eof

:log_warning
echo %YELLOW%[WARNING]%NC% %~1
goto :eof

:log_error
echo %RED%[ERROR]%NC% %~1
goto :eof

:: 默认配置
set "PLATFORM=windows"
set "ARCH=x86_64"
set "MODE=release"
set "VERSION="
set "CLEAN=false"
set "TEST=false"
set "SIGN=false"
set "DEPLOY=false"

:: 获取版本号
for /f "tokens=2 delims=:" %%a in ('node -p "require('./package.json').version" 2^>nul') do set "VERSION=%%a"
set "VERSION=%VERSION: =%"

if "%VERSION%"=="" set "VERSION=1.0.0"

:: 解析命令行参数
:parse_args
if "%~1"=="" goto :main
if /i "%~1"=="-p" set "PLATFORM=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="--platform" set "PLATFORM=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="-a" set "ARCH=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="--arch" set "ARCH=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="-m" set "MODE=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="--mode" set "MODE=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="-v" set "VERSION=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="--version" set "VERSION=%~2" & shift & shift & goto :parse_args
if /i "%~1"=="-c" set "CLEAN=true" & shift & goto :parse_args
if /i "%~1"=="--clean" set "CLEAN=true" & shift & goto :parse_args
if /i "%~1"=="-t" set "TEST=true" & shift & goto :parse_args
if /i "%~1"=="--test" set "TEST=true" & shift & goto :parse_args
if /i "%~1"=="-s" set "SIGN=true" & shift & goto :parse_args
if /i "%~1"=="--sign" set "SIGN=true" & shift & goto :parse_args
if /i "%~1"=="-d" set "DEPLOY=true" & shift & goto :parse_args
if /i "%~1"=="--deploy" set "DEPLOY=true" & shift & goto :parse_args
if /i "%~1"=="-h" goto :show_help
if /i "%~1"=="--help" goto :show_help
call :log_error "未知选项: %~1"
goto :show_help

:show_help
echo.
echo C-drive空间管理器 Windows构建脚本
echo.
echo 用法: %0 [选项]
echo.
echo 选项:
echo     -p, --platform [platform]   目标平台 (windows, all) [默认: windows]
echo     -a, --arch [architecture]   架构 (x86_64) [默认: x86_64]
echo     -m, --mode [mode]           构建模式 (debug, release) [默认: release]
echo     -v, --version [version]     版本号 [默认: package.json中的版本]
echo     -c, --clean                 清理构建缓存
echo     -t, --test                  运行测试
echo     -s, --sign                  代码签名
echo     -d, --deploy                部署到发布服务器
echo     -h, --help                  显示帮助信息
echo.
echo 示例:
echo     %0                          # 构建Windows发布版本
echo     %0 -s                       # 构建并签名
echo     %0 -m debug                 # 构建调试版本
echo     %0 -c -t                    # 清理缓存并运行测试
echo.
exit /b 0

:: 检查依赖
:check_dependencies
call :log_info "检查构建依赖..."

:: 检查Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "Node.js 未安装"
    exit /b 1
)

:: 检查Rust
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "Rust 未安装"
    exit /b 1
)

:: 检查pnpm
where pnpm >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "pnpm 未安装"
    exit /b 1
)

:: 检查Tauri CLI
where tauri >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "Tauri CLI 未安装"
    exit /b 1
)

call :log_success "所有依赖检查通过"
goto :eof

:: 清理构建缓存
:clean_build
if "%CLEAN%"=="true" (
    call :log_info "清理构建缓存..."
    
    :: 清理前端构建缓存
    if exist dist rd /s /q dist
    if exist "node_modules\.vite" rd /s /q "node_modules\.vite"
    if exist "node_modules\.cache" rd /s /q "node_modules\.cache"
    
    :: 清理Rust构建缓存
    cd src-tauri
    cargo clean
    cd ..
    
    :: 清理Tauri缓存
    if exist "src-tauri\target" rd /s /q "src-tauri\target"
    
    call :log_success "构建缓存清理完成"
)
goto :eof

:: 安装依赖
:install_dependencies
call :log_info "安装项目依赖..."

:: 安装前端依赖
pnpm install --frozen-lockfile

:: 安装Rust依赖
cd src-tauri
cargo fetch
cd ..

call :log_success "依赖安装完成"
goto :eof

:: 运行测试
:run_tests
if "%TEST%"=="true" (
    call :log_info "运行测试套件..."
    
    :: 前端测试
    pnpm run test
    
    :: Rust测试
    cd src-tauri
    cargo test
    cd ..
    
    :: 集成测试
    pnpm run test:integration
    
    call :log_success "测试运行完成"
)
goto :eof

:: 构建前端
:build_frontend
call :log_info "构建前端应用..."

:: 使用生产环境配置
set NODE_ENV=production

if "%MODE%"=="release" (
    pnpm run build:prod
) else (
    pnpm run build
)

call :log_success "前端构建完成"
goto :eof

:: 构建Rust后端
:build_backend
call :log_info "构建Rust后端..."

cd src-tauri

if "%MODE%"=="release" (
    :: 使用生产环境配置
    copy /y Cargo.prod.toml Cargo.toml
    copy /y tauri.conf.prod.json tauri.conf.json
    
    :: 发布构建
    cargo build --release
) else (
    :: 调试构建
    cargo build
)

cd ..

call :log_success "Rust后端构建完成"
goto :eof

:: 代码签名
:sign_code
if "%SIGN%"=="true" (
    call :log_info "进行代码签名..."
    
    :: 检查签名证书
    if "%CODE_SIGN_CERT%"=="" (
        call :log_warning "未设置代码签名证书，跳过签名"
        goto :eof
    )
    
    :: 签名可执行文件
    cd src-tauri\target\release\bundle\msi
    signtool sign /f "%CODE_SIGN_CERT%" /p "%CODE_SIGN_PASSWORD%" /t http://timestamp.digicert.com /v *.msi
    cd ..\..\..\..
    
    call :log_success "代码签名完成"
)
goto :eof

:: 打包应用
:package_app
call :log_info "打包应用程序..."

:: 使用Tauri打包
if "%PLATFORM%"=="all" (
    pnpm tauri build --target x86_64-pc-windows-msvc
) else (
    pnpm tauri build --target x86_64-pc-windows-msvc
)

call :log_success "应用打包完成"
goto :eof

:: 生成校验和
:generate_checksums
call :log_info "生成文件校验和..."

cd src-tauri\target\release\bundle

:: 为所有安装包生成校验和
for %%f in (*.msi *.dmg *.deb *.rpm *.AppImage) do (
    certutil -hashfile "%%f" SHA256 > "%%f.sha256"
    call :log_info "生成校验和: %%f.sha256"
)

cd ..\..\..\..

call :log_success "校验和生成完成"
goto :eof

:: 部署到发布服务器
:deploy_to_server
if "%DEPLOY%"=="true" (
    call :log_info "部署到发布服务器..."
    
    :: 检查部署配置
    if "%DEPLOY_SERVER%"=="" (
        call :log_error "未设置部署服务器配置"
        exit /b 1
    )
    
    :: 上传文件
    cd src-tauri\target\release\bundle
    xcopy /s /y . "%DEPLOY_PATH%\"
    cd ..\..\..\..
    
    call :log_success "部署完成"
)
goto :eof

:: 生成发布说明
:generate_release_notes
call :log_info "生成发布说明..."

> RELEASE_NOTES.md (
echo # C-drive空间管理器 v%VERSION% 发布说明
echo.
echo ## 版本信息
echo - **版本**: %VERSION%
echo - **构建时间**: %date% %time%
echo - **构建平台**: %PLATFORM%
echo - **架构**: %ARCH%
echo - **构建模式**: %MODE%
echo.
echo ## 系统要求
echo - **操作系统**: Windows 10/11
echo - **内存**: 4GB RAM ^(推荐8GB^)
echo - **磁盘空间**: 100MB可用空间
echo - **权限**: 管理员权限^（推荐^)
echo.
echo ## 安装包
dir /b src-tauri\target\release\bundle\*.msi 2>nul
echo.
echo ## 校验和
type src-tauri\target\release\bundle\*.sha256 2>nul
echo.
echo ## 更新内容
echo 请查看 CHANGELOG.md 获取详细的更新内容。
echo.
echo ## 技术支持
echo - **文档**: https://docs.dir-mover.com
echo - **支持邮箱**: support@dir-mover.com
echo - **社区论坛**: https://forum.dir-mover.com
echo.
echo ---
echo Copyright © 2024 Dir-Mover Technologies. All rights reserved.
)

call :log_success "发布说明生成完成"
goto :eof

:: 主构建流程
:main
call :log_info "开始构建 C-drive空间管理器 v%VERSION%"
call :log_info "目标平台: %PLATFORM%, 架构: %ARCH%, 模式: %MODE%"

:: 检查依赖
call :check_dependencies

:: 清理缓存（如果需要）
call :clean_build

:: 安装依赖
call :install_dependencies

:: 运行测试（如果需要）
call :run_tests

:: 构建前端
call :build_frontend

:: 构建后端
call :build_backend

:: 打包应用
call :package_app

:: 代码签名（如果需要）
call :sign_code

:: 生成校验和
call :generate_checksums

:: 生成发布说明
call :generate_release_notes

:: 部署（如果需要）
call :deploy_to_server

call :log_success "构建完成！"
call :log_info "安装包位置: src-tauri\target\release\bundle\"
call :log_info "发布说明: RELEASE_NOTES.md"

exit /b 0