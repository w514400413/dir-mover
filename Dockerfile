# C-drive空间管理器 构建环境容器
# 多阶段构建优化

# 阶段1: 前端构建
FROM node:18-alpine AS frontend-builder

# 设置工作目录
WORKDIR /app

# 安装系统依赖
RUN apk add --no-cache python3 make g++

# 复制包管理文件
COPY package.json pnpm-lock.yaml ./

# 安装pnpm
RUN npm install -g pnpm

# 安装前端依赖
RUN pnpm install --frozen-lockfile

# 复制前端源码
COPY . .

# 构建前端应用
RUN pnpm run build:prod

# 阶段2: Rust构建环境
FROM rust:1.75-alpine AS rust-builder

# 安装系统依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    curl \
    wget \
    git

# 安装Tauri依赖
RUN apk add --no-cache \
   webkit2gtk-dev \
    openssl-dev \
    curl \
    wget \
    file \
    libx11-dev \
    libxcb-dev \
    libxrandr-dev \
    libxext-dev \
    libxfixes-dev \
    libxi-dev \
    libxcursor-dev \
    libxdamage-dev \
    libxinerama-dev \
    libxss-dev \
    libxtst-dev \
    libgconf-2-4 \
    libnss3 \
    libasound2 \
    libxtst6 \
    libxrandr2 \
    libxss1 \
    libxcursor1 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libcups2 \
    libdrm2 \
    libxkbcommon0 \
    libxshmfence1 \
    libgbm1 \
    libpango1.0-0 \
    libgdk-pixbuf2.0-0 \
    libgtk-3-0 \
    libx11-xcb1 \
    libxcb-dri3-0 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shm0 \
    libxcb-xfixes0 \
    libxcb-shape0 \
    libxcb-xkb1 \
    libxcb-randr0 \
    libxcb-image0 \
    libxcb-util1 \
    libxcb-keysyms1 \
    libxcb-icccm4 \
    libxcb-cursor0 \
    libxcb-xinerama0 \
    libxcb-xinput0 \
    libxcb-xrm0 \
    libxcb-dri2-0 \
    libxcb-glx0 \
    libxcb-present0 \
    libxcb-sync1 \
    libxcb-render-util0 \
    libxcb-damage0 \
    libxcb-record0 \
    libxcb-res0 \
    libxcb-screensaver0 \
    libxcb-xf86dri0 \
    libxcb-xtest0 \
    libxcb-xv0 \
    libxcb-xvmc0

# 设置Rust环境
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# 安装Tauri CLI
RUN cargo install tauri-cli

# 设置工作目录
WORKDIR /app

# 复制Cargo文件
COPY src-tauri/Cargo.prod.toml src-tauri/Cargo.toml
COPY src-tauri/tauri.conf.prod.json src-tauri/tauri.conf.json

# 复制源码
COPY src-tauri/src ./src-tauri/src
COPY src-tauri/build.rs ./src-tauri/build.rs
COPY src-tauri/icons ./src-tauri/icons
COPY src-tauri/capabilities ./src-tauri/capabilities

# 构建Rust应用
RUN cd src-tauri && cargo build --release

# 阶段3: 最终构建阶段
FROM node:18-alpine AS final-builder

# 安装Tauri依赖
RUN apk add --no-cache \
    webkit2gtk \
    openssl \
    curl \
    wget \
    file \
    libx11 \
    libxcb \
    libxrandr \
    libxext \
    libxfixes \
    libxi \
    libxcursor \
    libxdamage \
    libxinerama \
    libxss \
    libxtst \
    libgconf-2-4 \
    libnss3 \
    libasound2 \
    libxtst6 \
    libxrandr2 \
    libxss1 \
    libxcursor1 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libcups2 \
    libdrm2 \
    libxkbcommon0 \
    libxshmfence1 \
    libgbm1 \
    libpango1.0-0 \
    libgdk-pixbuf2.0-0 \
    libgtk-3-0 \
    libx11-xcb1 \
    libxcb-dri3-0 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shm0 \
    libxcb-xfixes0 \
    libxcb-shape0 \
    libxcb-xkb1 \
    libxcb-randr0 \
    libxcb-image0 \
    libxcb-util1 \
    libxcb-keysyms1 \
    libxcb-icccm4 \
    libxcb-cursor0 \
    libxcb-xinerama0 \
    libxcb-xinput0 \
    libxcb-xrm0 \
    libxcb-dri2-0 \
    libxcb-glx0 \
    libxcb-present0 \
    libxcb-sync1 \
    libxcb-render-util0 \
    libxcb-damage0 \
    libxcb-record0 \
    libxcb-res0 \
    libxcb-screensaver0 \
    libxcb-xf86dri0 \
    libxcb-xtest0 \
    libxcb-xv0 \
    libxcb-xvmc0

# 安装pnpm
RUN npm install -g pnpm

# 设置工作目录
WORKDIR /app

# 复制前端构建结果
COPY --from=frontend-builder /app/dist ./dist

# 复制Rust构建结果
COPY --from=rust-builder /app/src-tauri/target/release/bundle ./bundle

# 复制构建脚本
COPY scripts/build.sh ./scripts/
COPY scripts/build.bat ./scripts/

# 设置执行权限
RUN chmod +x scripts/build.sh

# 创建输出目录
RUN mkdir -p /app/output

# 设置环境变量
ENV NODE_ENV=production
ENV TAURI_ENV=production

# 默认命令
CMD ["sh", "-c", "echo 'C-drive空间管理器构建环境已准备就绪'"]

# 标签
LABEL maintainer="Dir-Mover Technologies <support@dir-mover.com>"
LABEL version="1.0.0"
LABEL description="C-drive空间管理器构建环境"
LABEL org.opencontainers.image.title="C-drive空间管理器构建环境"
LABEL org.opencontainers.image.description="专业的C盘空间管理工具构建环境"
LABEL org.opencontainers.image.vendor="Dir-Mover Technologies"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/dir-mover/c-drive-space-manager"
LABEL org.opencontainers.image.documentation="https://docs.dir-mover.com"