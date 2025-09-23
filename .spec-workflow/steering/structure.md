# Project Structure

## Directory Organization

```
e:/rust/dir-mover/
├── src/                          # Frontend source code (Vue + TypeScript)
│   ├── components/               # Vue components
│   │   ├── ContextMenu.vue      # Right-click context menu
│   │   ├── DirectoryTree.vue    # Main directory tree display
│   │   ├── ErrorRecoveryMonitor.vue # Error handling UI
│   │   ├── MigrationDialog.vue  # Folder migration interface
│   │   ├── OperationLogViewer.vue # Log viewing component
│   │   ├── PerformanceMonitor.vue # Performance metrics display
│   │   └── TestManager.vue      # Testing interface
│   ├── services/                 # Frontend services and APIs
│   │   └── api.ts               # Tauri backend API wrapper
│   ├── types/                    # TypeScript type definitions
│   │   └── directory.ts         # Directory-related types
│   ├── assets/                   # Static assets
│   │   └── vue.svg              # Vue logo
│   ├── main.ts                  # Application entry point
│   ├── App.vue                  # Root Vue component
│   └── vite-env.d.ts            # Vite environment types
│
├── src-tauri/                    # Backend source code (Rust)
│   ├── src/                     # Rust source files
│   │   ├── lib.rs               # Main library entry point with Tauri commands
│   │   ├── main.rs              # Application main entry point
│   │   ├── disk_analyzer.rs     # Disk space analysis engine
│   │   ├── file_operations.rs   # File system operations
│   │   ├── migration_service.rs # Folder migration logic
│   │   ├── error_recovery.rs    # Error handling and recovery
│   │   ├── operation_logger.rs  # Operation logging system
│   │   ├── performance_optimizer.rs # Performance optimization
│   │   ├── logger.rs            # Logging configuration
│   │   ├── types.rs             # Shared type definitions
│   │   └── tests/               # Test modules
│   │       ├── mod.rs            # Test module declarations
│   │       ├── unit_tests.rs     # Unit tests
│   │       ├── integration_tests.rs # Integration tests
│   │       ├── e2e_tests.rs      # End-to-end tests
│   │       ├── performance_tests.rs # Performance benchmarks
│   │       └── test_utils.rs     # Test utilities and helpers
│   ├── Cargo.toml               # Rust package configuration
│   ├── Cargo.prod.toml          # Production build configuration
│   ├── build.rs                 # Build script
│   ├── tauri.conf.json          # Tauri configuration
│   ├── tauri.conf.prod.json     # Production Tauri configuration
│   ├── capabilities/            # Tauri capability definitions
│   │   └── default.json         # Default security capabilities
│   ├── icons/                   # Application icons
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── icon.icns            # macOS icon
│   │   ├── icon.ico             # Windows icon
│   │   └── icon.png             # Generic icon
│   └── gen/                     # Generated files (schemas, etc.)
│
├── docs/                        # Documentation
│   ├── README.md                # Documentation index
│   ├── quick_start.md           # Quick start guide
│   ├── user_manual.md           # Detailed user manual
│   ├── operation_guide.md       # Operation procedures
│   ├── deployment_guide.md      # Deployment instructions
│   ├── error_codes.md           # Error code reference
│   └── faq.md                   # Frequently asked questions
│
├── scripts/                     # Build and utility scripts
│   ├── build.sh                 # Unix build script
│   └── build.bat                # Windows build script
│
├── public/                      # Static public assets
│   ├── vite.svg                 # Vite logo
│   └── tauri.svg                # Tauri logo
│
├── .github/                     # GitHub configuration
│   └── workflows/               # CI/CD workflows
│
├── .vscode/                     # VS Code configuration
│   ├── extensions.json          # Recommended extensions
│   └── settings.json            # Workspace settings
│
├── .spec-workflow/              # Specification workflow
│   ├── templates/               # Document templates
│   ├── steering/                # Steering documents (this file)
│   └── specs/                   # Feature specifications
│
├── package.json                 # Node.js package configuration
├── pnpm-lock.yaml               # pnpm lock file
├── pnpm-workspace.yaml          # pnpm workspace configuration
├── tsconfig.json                # TypeScript configuration
├── tsconfig.node.json           # TypeScript node configuration
├── vite.config.ts               # Vite development configuration
├── vite.config.prod.ts          # Vite production configuration
├── rust-toolchain.toml          # Rust toolchain specification
├── Dockerfile                   # Container configuration
├── docker-compose.yml           # Docker compose setup
├── .gitignore                   # Git ignore rules
├── README.md                    # Project README
└── ARCHITECTURE.md              # Technical architecture document
```

## Naming Conventions

### Files
- **Vue Components**: PascalCase with `.vue` extension (e.g., `DirectoryTree.vue`)
- **TypeScript Files**: camelCase with `.ts` extension (e.g., `api.ts`)
- **Rust Modules**: snake_case with `.rs` extension (e.g., `disk_analyzer.rs`)
- **Test Files**: Suffix with `_test.rs` or `.test.ts` (e.g., `unit_tests.rs`)
- **Configuration**: kebab-case with appropriate extension (e.g., `tauri.conf.json`)

### Code
- **Vue Components**: PascalCase for component names
- **TypeScript/Interfaces**: PascalCase for types, camelCase for functions
- **Rust Types**: PascalCase for structs/enums, snake_case for functions
- **Constants**: UPPER_SNAKE_CASE for both languages
- **Variables**: camelCase (TypeScript), snake_case (Rust)

## Import Patterns

### Frontend Import Order
1. Vue core imports (`vue`, `vue-router`)
2. Third-party libraries (`element-plus`, `pinia`)
3. Internal utilities (`@/utils/*`)
4. Components (`@/components/*`)
5. Types (`@/types/*`)
6. Relative imports for local files
7. Style imports

### Backend Import Order (Rust)
1. Standard library (`std::*`)
2. External crates (`tokio`, `serde`)
3. Internal modules (`crate::*`)
4. Local module imports (`super::*`, `self::*`)

### Module Organization
- **Frontend**: Absolute imports from `@/` alias pointing to `src/`
- **Backend**: Crate-relative imports using `crate::` prefix
- **Shared Types**: Common types defined in both `src/types/` and `src-tauri/src/types.rs`

## Code Structure Patterns

### Vue Component Structure
```vue
<template>
  <!-- Component template -->
</template>

<script setup lang="ts">
// Imports
import { ref, computed, onMounted } from 'vue'
import type { DirectoryInfo } from '@/types/directory'

// Props definition
interface Props {
  path: string
}
const props = defineProps<Props>()

// Emits definition
const emit = defineEmits<{
  select: [path: string]
}>()

// Reactive state
const directory = ref<DirectoryInfo | null>(null)
const loading = ref(false)

// Computed properties
const sizeFormatted = computed(() => formatSize(directory.value?.size || 0))

// Lifecycle hooks
onMounted(() => {
  loadDirectory()
})

// Methods
async function loadDirectory() {
  loading.value = true
  try {
    directory.value = await api.scanDirectory(props.path)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* Component styles */
</style>
```

### Rust Module Structure
```rust
// Imports
use std::path::{Path, PathBuf};
use tokio::fs;
use serde::{Serialize, Deserialize};

// Type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: PathBuf,
    pub size: u64,
    pub subdirectories: Vec<DirectoryInfo>,
}

// Constants
const MAX_SCAN_DEPTH: usize = 10;

// Public API functions
pub async fn scan_directory(path: &Path) -> Result<DirectoryInfo, Error> {
    // Implementation
}

// Private helper functions
async fn calculate_directory_size(path: &Path) -> Result<u64, Error> {
    // Implementation
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path not found: {0}")]
    NotFound(String),
}
```

### File Organization Principles
1. **Single Responsibility**: Each module handles one specific concern
2. **Feature Grouping**: Related functionality grouped in same directory
3. **Interface Segregation**: Clear separation between public API and implementation
4. **Dependency Inversion**: Dependencies on abstractions, not concrete implementations

## Code Organization Principles

1. **Separation of Concerns**
   - Frontend: UI rendering and user interaction
   - Backend: File system operations and business logic
   - Shared: Type definitions and validation rules

2. **Modularity**
   - Self-contained components with clear interfaces
   - Reusable utilities and helper functions
   - Plugin-ready architecture for future extensions

3. **Testability**
   - Dependency injection for easy mocking
   - Pure functions where possible
   - Clear separation of I/O and business logic

4. **Maintainability**
   - Comprehensive documentation and comments
   - Consistent coding standards
   - Automated formatting and linting

## Module Boundaries

### Frontend Boundaries
- **Components**: Pure UI components with minimal business logic
- **Services**: API communication and data transformation
- **Types**: Shared interfaces and type definitions
- **Utils**: Pure utility functions

### Backend Boundaries
- **Commands**: Tauri command handlers (public API)
- **Services**: Business logic implementation
- **Types**: Rust data structures and error types
- **Utils**: Helper functions and common operations

### Cross-Cutting Concerns
- **Error Handling**: Consistent error types and messages
- **Logging**: Structured logging throughout the application
- **Configuration**: Centralized configuration management
- **Security**: Input validation and permission checks

## Code Size Guidelines

### File Size Limits
- **Vue Components**: Maximum 300 lines (template + script + style)
- **TypeScript Modules**: Maximum 500 lines
- **Rust Modules**: Maximum 1000 lines
- **Test Files**: Maximum 200 lines per test case

### Function Complexity
- **Maximum Parameters**: 4 parameters per function
- **Maximum Nesting**: 3 levels of indentation
- **Cyclomatic Complexity**: Maximum 10 for critical paths
- **Function Length**: Maximum 50 lines for complex logic

### Component Complexity
- **Maximum Props**: 10 props per Vue component
- **Maximum State**: 5 reactive variables per component
- **Maximum Methods**: 10 methods per component
- **Maximum Watchers**: 3 watchers per component

## Dashboard/Monitoring Structure

### Component Hierarchy
```
src/components/
├── App.vue                      # Root component with layout
├── DirectoryTree.vue            # Main content area
│   ├── TreeNode.vue             # Individual directory nodes
│   └── TreeToolbar.vue          # Tree operations toolbar
├── MigrationDialog.vue          # Migration workflow
│   ├── MigrationSteps.vue       # Step-by-step process
│   └── MigrationSummary.vue     # Results summary
├── PerformanceMonitor.vue       # Real-time metrics
│   ├── MetricCard.vue           # Individual metric display
│   └── MetricChart.vue          # Historical charts
└── ErrorRecoveryMonitor.vue     # Error handling UI
    ├── ErrorList.vue            # Error list display
    └── RecoveryActions.vue      # Recovery options
```

### State Management Pattern
- **Local State**: Component-level reactive data
- **Shared State**: Pinia stores for cross-component data
- **Backend State**: Rust-side state accessed via Tauri commands
- **Persistent State**: Configuration and user preferences

### Communication Patterns
- **Parent-Child**: Props down, events up
- **Sibling Components**: Shared state via Pinia
- **Frontend-Backend**: Tauri command invocation
- **Real-time Updates**: Tauri event system

## Documentation Standards

### Code Documentation
- **Public APIs**: JSDoc for TypeScript, rustdoc for Rust
- **Complex Logic**: Inline comments explaining algorithms
- **Business Rules**: Comments explaining why, not what
- **Error Cases**: Documentation of error conditions and handling

### Component Documentation
- **Props**: Detailed prop types and usage
- **Events**: Documented emit patterns
- **Slots**: Slot content and scope documentation
- **Examples**: Usage examples in documentation

### Module Documentation
- **README.md**: Each major directory with overview
- **API Documentation**: Auto-generated from code comments
- **Architecture Docs**: High-level design decisions
- **Change Log**: Version history and breaking changes

### Testing Documentation
- **Test Plans**: Documented testing strategies
- **Test Cases**: Clear descriptions of what is being tested
- **Mock Data**: Documented test data and scenarios
- **Performance Benchmarks**: Baseline performance expectations