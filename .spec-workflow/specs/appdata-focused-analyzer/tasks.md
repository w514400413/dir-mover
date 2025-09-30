# AppData Focused Analyzer - Tasks

## 修改现有任务

- [x] 1. 创建AppData分析器Rust模块
  - File: src-tauri/src/appdata_analyzer.rs
  - 实现AppData路径检测和扫描逻辑
  - 重用现有DiskAnalyzer引擎
  - Purpose: 提供专门的AppData目录分析功能
  - _Leverage: src-tauri/src/disk_analyzer.rs, src-tauri/src/types.rs_
  - _Requirements: FR-1, FR-2, FR-3_
  - _Prompt: Role: Rust Developer specializing in Windows file system operations | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Create AppData analyzer module in src-tauri/src/appdata_analyzer.rs following requirements FR-1, FR-2, FR-3, leveraging existing DiskAnalyzer from src-tauri/src/disk_analyzer.rs and types from src-tauri/src/types.rs | Restrictions: Must reuse existing DiskAnalyzer engine, follow Rust naming conventions, handle Windows-specific path detection, ensure error handling for permission issues | Success: Module correctly detects AppData paths, scans only Local/LocalLow/Roaming directories, filters 1GB+ folders, integrates with existing error handling_

- [x] 2. 添加AppData扫描Tauri命令
  - File: src-tauri/src/lib.rs (modify existing)
  - 在现有命令系统中添加scan_appdata命令
  - 集成AppDataAnalyzer到Tauri命令体系
  - Purpose: 通过Tauri IPC暴露AppData扫描功能
  - _Leverage: src-tauri/src/lib.rs, src-tauri/src/appdata_analyzer.rs_
  - _Requirements: FR-2, FR-3, FR-4_
  - _Prompt: Role: Tauri Developer with expertise in Rust backend commands | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Add scan_appdata command to src-tauri/src/lib.rs following requirements FR-2, FR-3, FR-4, leveraging existing command patterns and AppDataAnalyzer | Restrictions: Must follow existing Tauri command patterns, maintain async/await consistency, provide proper error responses, integrate with existing logging | Success: New command is properly registered, handles async scanning, returns formatted results, includes error handling and progress reporting_

- [x] 3. 创建AppData类型定义
  - File: src/types/appdata.ts
  - 定义AppData扫描相关的TypeScript接口
  - 扩展现有DirectoryInfo类型
  - Purpose: 提供类型安全的AppData数据结构
  - _Leverage: src/types/directory.ts_
  - _Requirements: FR-3, FR-4_
  - _Prompt: Role: TypeScript Developer specializing in type systems | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Create AppData type definitions in src/types/appdata.ts following requirements FR-3, FR-4, extending existing DirectoryInfo from src/types/directory.ts | Restrictions: Must maintain compatibility with existing types, follow project naming conventions, provide comprehensive type coverage, include proper documentation | Success: All AppData-related interfaces are defined, extend existing types correctly, provide full type safety for frontend development_

- [x] 4. 添加AppData API服务方法
  - File: src/services/api.ts (modify existing)
  - 在现有API服务中添加scanAppData方法
  - 集成AppData扫描功能到前端服务层
  - Purpose: 提供前端调用的AppData扫描API
  - _Leverage: src/services/api.ts, src/types/appdata.ts_
  - _Requirements: FR-2, FR-3, FR-4_
  - _Prompt: Role: Frontend Developer with expertise in API integration | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Add scanAppData method to src/services/api.ts following requirements FR-2, FR-3, FR-4, leveraging existing API patterns and AppData types | Restrictions: Must follow existing API service patterns, maintain error handling consistency, provide proper TypeScript typing, integrate with existing logging | Success: API method correctly calls Tauri command, handles responses and errors, returns properly typed data, includes progress handling_

## 新需求和修改任务

- [x] 5. 修改AppData分析器后端逻辑
  - File: src-tauri/src/appdata_analyzer.rs (modify existing)
  - 修改扫描逻辑以获取一级子目录和文件
  - 实现动态排序功能
  - 添加迁移支持功能
  - Purpose: 支持新的AppData分析需求
  - _Leverage: src-tauri/src/disk_analyzer.rs, src-tauri/src/migration_service.rs_
  - _Requirements: FR-2, FR-3, FR-4, FR-5_
  - _Prompt: Role: Rust Developer specializing in Windows file system operations | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Modify AppData analyzer in src-tauri/src/appdata_analyzer.rs following requirements FR-2, FR-3, FR-4, FR-5, leveraging existing DiskAnalyzer and MigrationService | Restrictions: Must scan first-level items under Local/LocalLow/Roaming directories, provide dynamic sorting during scan, support migration operations, maintain compatibility with existing code | Success: Analyzer correctly identifies first-level items, provides real-time sorting, supports migration operations, integrates with existing services, maintains performance requirements_

- [x] 6. 修改AppData扫描Vue组件
  - File: src/components/AppDataScanner.vue (modify existing)
  - 修改界面以显示三个主要目录下的一级子目录和文件
  - 实现动态排序功能（扫描过程中实时排序）
  - 添加目标盘符选择功能
  - Purpose: 提供用户友好的AppData分析界面，支持新需求
  - _Leverage: src/components/DirectoryTree.vue, Element Plus组件库_
  - _Requirements: FR-3, FR-4, FR-5, FR-6, UX-1, UX-2_
  - _Prompt: Role: Vue Developer specializing in UI components and user experience | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Modify AppData scanner component in src/components/AppDataScanner.vue following requirements FR-3, FR-4, FR-5, FR-6, UX-1, UX-2, leveraging DirectoryTree patterns and Element Plus components | Restrictions: Must display only first-level items under Local/LocalLow/Roaming directories, provide dynamic sorting during scan, add target drive selection, follow existing component patterns | Success: Component displays first-level items from three main directories, provides real-time sorting during scan, includes drive selection interface, handles all user interactions, integrates with existing theme and styling_

- [x] 7. 实现动态排序和1GB筛选功能
  - File: src/components/AppDataScanner.vue (continue from task 6)
  - 实现扫描过程中的动态逆序排序
  - 最终只显示1GB以上的子目录和文件
  - Purpose: 提供智能的结果展示和筛选
  - _Leverage: src/components/AppDataScanner.vue_
  - _Requirements: FR-3, FR-4, UX-2, UX-3_
  - _Prompt: Role: Frontend Developer with expertise in data manipulation and user interfaces | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Implement dynamic sorting and 1GB filtering functionality in src/components/AppDataScanner.vue following requirements FR-3, FR-4, UX-2, UX-3 | Restrictions: Must provide real-time sorting during scan, automatically filter to show only 1GB+ items after scan completion, maintain UI responsiveness, allow manual threshold adjustment | Success: Results are dynamically sorted by size (descending) during scan, 1GB+ filter applied after completion, interface remains responsive, user can adjust thresholds if needed_

- [x] 8. 添加AppData迁移功能
  - File: src/components/AppDataScanner.vue (add migration features)
  - 实现目标盘符选择对话框
  - 集成现有迁移逻辑进行文件迁移
  - 只迁移显示的大文件（1GB以上）
  - Purpose: 提供完整的AppData文件迁移功能
  - _Leverage: src/services/api.ts, src/types/appdata.ts_
  - _Requirements: FR-5, UX-3_
  - _Prompt: Role: Full-stack Developer with expertise in file operations and user interfaces | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Add AppData migration functionality to src/components/AppDataScanner.vue following requirements FR-5, UX-3, leveraging existing migration API and AppData types | Restrictions: Must provide drive selection interface, create same-name paths on target drive, use existing migration logic, only migrate items shown in UI (1GB+), provide migration progress feedback | Success: Users can select target drive, migration creates proper directory structure, uses existing migration service, only migrates large files, provides clear progress and success/failure feedback_

- [x] 9. 更新AppData类型定义
  - File: src/types/appdata.ts (modify existing)
  - 添加AppDataFirstLevelItem接口定义
  - 添加AppDataMigrationOptions接口定义
  - 更新现有的AppDataInfo接口
  - Purpose: 支持新的AppData分析功能
  - _Leverage: src/types/appdata.ts, src/types/directory.ts_
  - _Requirements: FR-3, FR-5_
  - _Prompt: Role: TypeScript Developer specializing in type systems | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Add new type definitions to src/types/appdata.ts following requirements FR-3, FR-5, extending existing types and maintaining compatibility | Restrictions: Must maintain backward compatibility, follow project naming conventions, provide comprehensive type coverage for first-level items and migration options, include proper documentation | Success: New interfaces are defined correctly, existing types are updated appropriately, provide full type safety for new features, maintain compatibility with existing code_

- [x] 10. 添加迁移API服务方法
  - File: src/services/api.ts (modify existing)
  - 添加AppData迁移相关API方法
  - 集成目标盘符选择功能
  - 提供迁移进度反馈
  - Purpose: 提供完整的AppData迁移API支持
  - _Leverage: src/services/api.ts, src/types/appdata.ts, src/types/directory.ts_
  - _Requirements: FR-5, UX-3_
  - _Prompt: Role: Frontend Developer with expertise in API integration and file operations | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Add AppData migration API methods to src/services/api.ts following requirements FR-5, UX-3, leveraging existing API patterns and new AppData types | Restrictions: Must follow existing API service patterns, integrate with migration service, provide proper error handling, support progress reporting, maintain TypeScript typing | Success: API methods support AppData migration operations, integrate with target drive selection, provide progress feedback, handle errors appropriately, maintain type safety_

- [x] 11. 集成AppData扫描器到主应用
  - File: src/App.vue (modify existing)
  - 在主界面中添加AppData扫描器入口
  - 配置路由和导航
  - Purpose: 将修改后的AppData功能集成到现有应用中
  - _Leverage: src/App.vue, src/components/AppDataScanner.vue_
  - _Requirements: UX-1, UX-3_
  - _Prompt: Role: Full-stack Developer with expertise in application integration | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Integrate modified AppData scanner into src/App.vue following requirements UX-1, UX-3, leveraging existing app structure and modified AppDataScanner component | Restrictions: Must maintain existing app functionality, provide clear navigation to AppData feature, follow existing routing patterns, ensure seamless user experience with new features | Success: Modified AppData scanner is accessible from main interface, navigation works smoothly, doesn't interfere with existing features, provides intuitive user flow with migration capabilities_

- [x] 12. 修改单元测试
  - File: src-tauri/src/tests/appdata_analyzer_tests.rs (modify existing)
  - 测试一级子目录和文件检测逻辑
  - 验证动态排序功能
  - 测试新的迁移支持功能
  - Purpose: 确保修改后的AppData分析器可靠性
  - _Leverage: src-tauri/src/tests/test_utils.rs, src-tauri/src/appdata_analyzer.rs_
  - _Requirements: SC-4, NFR-2_
  - _Prompt: Role: QA Engineer with expertise in Rust testing and file system operations | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Modify unit tests in src-tauri/src/tests/appdata_analyzer_tests.rs following requirements SC-4, NFR-2, leveraging test utilities and modified AppDataAnalyzer | Restrictions: Must test first-level item detection, dynamic sorting functionality, migration support, test core functionality without relying on actual file system, use mocking where appropriate, cover error scenarios | Success: All modified AppDataAnalyzer methods are tested, first-level item detection is verified, dynamic sorting is validated, migration support is tested, error cases are covered, tests run reliably_

- [x] 13. 更新集成测试
  - File: src-tauri/src/tests/integration_tests.rs (modify existing)
  - 更新AppData扫描端到端测试
  - 验证新的扫描和迁移流程
  - Purpose: 确保修改后的各组件协同工作正常
  - _Leverage: src-tauri/src/tests/integration_tests.rs, src-tauri/src/lib.rs_
  - _Requirements: SC-1, SC-2, NFR-1_
  - _Prompt: Role: Integration Test Engineer with expertise in system testing | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Update AppData integration tests in src-tauri/src/tests/integration_tests.rs following requirements SC-1, SC-2, NFR-1, leveraging existing test infrastructure | Restrictions: Must test complete user workflows including migration, verify performance requirements, ensure data accuracy, handle various system configurations | Success: Integration tests cover modified AppData scanning and migration workflow, performance benchmarks are met, data accuracy is verified, tests pass on different Windows configurations_

- [x] 14. 性能优化和最终代码清理
  - Files: 所有修改的文件
  - 优化扫描算法性能
  - 清理代码和添加文档
  - 确保所有新功能正常工作
  - Purpose: 确保代码质量和性能达标
  - _Leverage: src-tauri/src/performance_optimizer.rs, src-tauri/src/logger.rs_
  - _Requirements: NFR-1, NFR-2, SC-4_
  - _Prompt: Role: Senior Developer with expertise in performance optimization and code quality | Task: Implement the task for spec appdata-focused-analyzer, first run spec-workflow-guide to get the workflow guide then implement the task: Perform final performance optimization and code cleanup across all modified files following requirements NFR-1, NFR-2, SC-4, leveraging performance optimizer and logging utilities | Restrictions: Must maintain code readability, follow Rust best practices, ensure no functional changes during optimization, add comprehensive documentation, verify all new features work correctly | Success: Scan performance meets <30s requirement, memory usage stays under 150MB, code is clean and well-documented, all tests continue to pass, all new features work correctly, performance benchmarks are documented_