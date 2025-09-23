# Technology Stack

## Project Type
C盘空间管理工具是一款桌面应用程序，采用Tauri + Rust + Vue + TypeScript技术栈开发的跨平台系统工具。该应用专注于Windows平台的磁盘空间分析和文件管理，提供高性能、安全可靠的文件夹迁移功能。

## Core Technologies

### Primary Language(s)
- **Frontend**: TypeScript 5.0+ with Vue 3 Composition API
- **Backend**: Rust 1.70+ (Edition 2021)
- **Runtime**: Tauri 1.5+ (Rust-based desktop application framework)
- **Build System**: Vite 4.0+ for frontend bundling, Cargo for Rust compilation

### Key Dependencies/Libraries

#### Frontend Dependencies
- **Vue 3.3+**: Progressive JavaScript framework with Composition API
- **Element Plus 2.4+**: Vue 3 UI component library
- **TypeScript 5.0+**: Type-safe JavaScript superset
- **Vite 4.0+**: Fast build tool and dev server
- **Pinia 2.1+**: Vue state management (if needed for complex state)
- **Vue Router 4.2+**: Client-side routing (for multi-view applications)

#### Backend Dependencies (Rust)
- **Tauri 1.5+**: Cross-platform desktop application framework
- **Tokio 1.35+**: Async runtime for high-performance I/O operations
- **Serde 1.0+**: Serialization framework for data exchange
- **Walkdir 2.4+**: Directory traversal with configurable options
- **Filetime 0.2+**: Cross-platform file time handling
- **Sysinfo 0.30+**: System information gathering
- **Log 0.4+**: Logging facade with multiple backend support
- **Anyhow 1.0+**: Flexible error handling
- **Thiserror 1.0+**: Derive-based error types

#### Development & Build Tools
- **pnpm 8.0+**: Fast, disk space efficient package manager
- **Rustup**: Rust toolchain manager
- **Tauri CLI**: Desktop app development and building
- **ESLint + Prettier**: Code formatting and linting
- **Rust Analyzer**: IDE support for Rust development

### Application Architecture
**Hybrid Desktop Architecture**: 
- Frontend: SPA (Single Page Application) built with Vue 3
- Backend: Rust-based Tauri application with command pattern
- Communication: IPC (Inter-Process Communication) via Tauri's invoke system
- State Management: Frontend-centric with backend state persistence
- Error Handling: Centralized error handling with user-friendly messages

### Data Storage
- **Configuration**: Tauri's built-in config system (JSON-based)
- **Cache**: Local file-based cache for scan results
- **Logs**: Rotating file logs with structured logging
- **Migration History**: SQLite database for operation tracking
- **Temporary Data**: System temp directory with cleanup

### External Integrations
- **File System APIs**: Windows File System APIs via Rust's std::fs
- **System APIs**: Windows Registry and WMI for system information
- **Shell Integration**: Windows Shell for context menu integration
- **Power Management**: Windows power APIs for background operation handling

### Monitoring & Dashboard Technologies
- **Dashboard Framework**: Vue 3 with Element Plus components
- **Real-time Communication**: Tauri's event system (WebSocket-like)
- **Visualization Libraries**: Element Plus charts, potential D3.js integration
- **State Management**: Vue's reactive system with composables
- **Progress Indication**: Native progress bars and status updates

## Development Environment

### Build & Development Tools
- **Development Server**: Vite dev server with HMR (Hot Module Replacement)
- **Build System**: Vite for frontend, Cargo for Rust backend
- **Package Management**: pnpm with workspace support
- **Development Workflow**: Concurrent frontend/backend development with Tauri dev mode

### Code Quality Tools
- **Static Analysis**: 
  - ESLint with TypeScript and Vue rules
  - Clippy for Rust linting
  - Tauri-specific linting rules
- **Formatting**: 
  - Prettier for TypeScript/Vue
  - rustfmt for Rust code
- **Testing Framework**: 
  - Vitest for unit testing (Vue/TypeScript)
  - Rust's built-in test framework
  - Tauri's testing utilities for integration tests
- **Documentation**: 
  - TypeDoc for TypeScript API docs
  - rustdoc for Rust documentation
  - mdBook for user documentation

### Version Control & Collaboration
- **VCS**: Git with conventional commits
- **Branching Strategy**: Git Flow with feature/develop/main branches
- **Code Review**: GitHub Pull Requests with required reviews
- **CI/CD**: GitHub Actions for automated testing and building

### Dashboard Development
- **Live Reload**: Vite HMR for instant frontend updates
- **Backend Hot Reload**: Cargo watch for Rust code changes
- **Debugging**: Chrome DevTools for frontend, VS Code for Rust
- **Performance Profiling**: Built-in browser dev tools and Rust profiling

## Deployment & Distribution

### Target Platform(s)
- **Primary**: Windows 10/11 (x64, ARM64)
- **Secondary**: macOS (Intel, Apple Silicon)
- **Tertiary**: Linux (Ubuntu, Fedora, Arch)

### Distribution Method
- **Microsoft Store**: Windows Package Manager (winget)
- **Direct Download**: GitHub Releases with MSI installers
- **Package Managers**: Homebrew (macOS), various Linux package managers
- **Portable**: ZIP distribution for portable usage

### Installation Requirements
- **Windows**: Windows 10 version 1903+ or Windows 11
- **macOS**: macOS 10.15+ (Catalina or later)
- **Linux**: GTK 3.24+ and WebKit2GTK 4.0+
- **Hardware**: 4GB RAM minimum, 100MB disk space

### Update Mechanism
- **Auto-updater**: Tauri's built-in updater with signature verification
- **Update Channels**: Stable, Beta, and Nightly channels
- **Rollback**: Automatic rollback on failed updates
- **Notifications**: System notifications for available updates

## Technical Requirements & Constraints

### Performance Requirements
- **Scan Performance**: 100GB directory scan in <30 seconds on SSD
- **Memory Usage**: <200MB peak memory during scanning
- **CPU Usage**: <10% CPU utilization during background operation
- **Startup Time**: <2 seconds cold start, <500ms warm start
- **Migration Speed**: 1GB/minute minimum on mechanical drives

### Compatibility Requirements
- **Platform Support**: Windows 10+, macOS 10.15+, Ubuntu 18.04+
- **Architecture**: x86_64, ARM64 (Apple Silicon, Windows on ARM)
- **Rust Version**: 1.70+ (MSRV - Minimum Supported Rust Version)
- **Node.js Version**: 18.0+ for development
- **Browser Engine**: WebKit2GTK (Linux), WebKit (macOS), WebView2 (Windows)

### Security & Compliance
- **Code Signing**: All binaries signed with trusted certificates
- **Sandboxing**: Tauri's built-in security sandbox
- **Permission Model**: Explicit user consent for file operations
- **Data Protection**: No telemetry without user consent
- **Vulnerability Management**: Regular dependency updates and security audits

### Scalability & Reliability
- **File System Scale**: Support for 1M+ files and 10TB+ directories
- **Concurrent Operations**: Thread-safe operations with proper locking
- **Error Recovery**: Graceful handling of permission errors, disk full, etc.
- **Crash Recovery**: Automatic recovery from unexpected crashes
- **Resource Limits**: Configurable memory and time limits for operations

## Technical Decisions & Rationale

### Decision Log

1. **Tauri vs Electron**
   - **Chosen**: Tauri for smaller bundle size, better performance, and Rust integration
   - **Rationale**: 10x smaller binary size, native performance, memory safety
   - **Trade-offs**: Smaller ecosystem, newer technology

2. **Rust for Backend**
   - **Chosen**: Rust for memory safety, performance, and excellent Windows support
   - **Rationale**: Zero-cost abstractions, fearless concurrency, strong type system
   - **Trade-offs**: Steeper learning curve, longer compile times

3. **Vue 3 vs React**
   - **Chosen**: Vue 3 for better TypeScript integration and simpler mental model
   - **Rationale**: Composition API clarity, excellent documentation, smaller bundle
   - **Trade-offs**: Smaller ecosystem than React, fewer developers

4. **Tokio Async Runtime**
   - **Chosen**: Tokio as the de facto standard for Rust async
   - **Rationale**: Mature ecosystem, excellent performance, wide adoption
   - **Trade-offs**: Added complexity, debugging challenges

5. **SQLite for History**
   - **Chosen**: SQLite for zero-configuration, reliable local storage
   - **Rationale**: ACID compliance, cross-platform, no server needed
   - **Trade-offs**: Not suitable for high-concurrency write operations

## Known Limitations

### Current Limitations
- **Windows Focus**: Primary optimization for Windows, other platforms may have feature gaps
- **Single User**: Designed for single-user scenarios, no multi-user support
- **Local Storage**: All data stored locally, no cloud synchronization
- **Admin Rights**: Some operations may require administrator privileges
- **File Locking**: Cannot move files that are locked by running applications

### Technical Debt
- **Hardcoded Paths**: Some Windows-specific paths need abstraction for cross-platform support
- **Error Handling**: Inconsistent error handling in some legacy code paths
- **Testing Coverage**: Integration test coverage could be improved
- **Documentation**: API documentation needs expansion
- **Performance**: Large directory scanning could be further optimized

### Future Improvements
- **Plugin Architecture**: Support for third-party extensions
- **Cloud Integration**: Optional cloud backup and synchronization
- **Advanced Analytics**: More sophisticated space usage analysis
- **Automation API**: Programmatic interface for enterprise automation
- **Mobile Companion**: Mobile app for remote monitoring