---
name: git-commit
description: 仅用 Git 分析改动并自动生成 conventional commit 信息（可选 emoji）；必要时建议拆分提交，默认运行本地 Git 钩子（可 --no-verify 跳过）
license: MIT
compatibility: opencode
metadata:
  audience: developers
  workflow: version-control
---
## What I do

该命令在**不依赖任何包管理器/构建工具**的前提下，仅通过 **Git**：

- 读取改动（staged/unstaged）
- 判断是否需要**拆分为多次提交**
- 为每个提交生成 **Conventional Commits** 风格的信息（可选 emoji）
- 按需执行 `git add` 与 `git commit`（默认运行本地 Git 钩子；可 `--no-verify` 跳过）

## When to use me

使用此命令当您需要：
- 快速生成符合 Conventional Commits 规范的提交信息
- 分析当前改动并获得提交建议
- 在不依赖额外工具的情况下完成 Git 提交
- 自动判断是否需要拆分提交

## My workflow

1. **仓库/分支校验**
   - 通过 `git rev-parse --is-inside-work-tree` 判断是否位于 Git 仓库
   - 读取当前分支/HEAD 状态；如处于 rebase/merge 冲突状态，先提示处理冲突后再继续

2. **改动检测**
   - 用 `git status --porcelain` 与 `git diff` 获取已暂存与未暂存的改动
   - 若已暂存文件为 0：
     - 若传入 `--all` → 执行 `git add -A`
     - 否则提示选择：继续仅分析未暂存改动并给出建议，或取消命令后手动分组暂存

3. **拆分建议（Split Heuristics）**
   - 按**关注点**、**文件模式**、**改动类型**聚类（示例：源代码 vs 文档、测试；不同目录/包；新增 vs 删除）
   - 若检测到**多组独立变更**或 diff 规模过大（如 > 300 行 / 跨多个顶级目录），建议拆分提交

4. **提交信息生成（Conventional 规范，可选 Emoji）**
   - 自动推断 `type`（`feat`/`fix`/`docs`/`refactor`/`test`/`chore`/`perf`/`style`/`ci`/`revert` …）与可选 `scope`
   - 生成消息头：`[<emoji>] <type>(<scope>)?: <subject>`（首行 ≤ 72 字符，祈使语气）
   - 生成消息体：使用列表格式，每项必须使用动词开头的祈使句
   - 生成消息脚注（如有 BREAKING CHANGE 或 trailer）
   - 根据 Git 历史提交的主要语言选择提交信息语言
   - 将草稿写入 `.git/COMMIT_EDITMSG`，并用于 `git commit`

5. **执行提交**
   - 单提交场景：`git commit [-S] [--no-verify] [-s] -F .git/COMMIT_EDITMSG`
   - 多提交场景：按分组给出明确指令；若允许执行则逐一完成

6. **安全回滚**
   - 如误暂存，可用 `git restore --staged <paths>` 撤回暂存

## Usage

```bash
/git-commit
/git-commit --no-verify
/git-commit --emoji
/git-commit --all --signoff
/git-commit --amend
/git-commit --scope ui --type feat --emoji
```

### Options

| 参数 | 说明 |
|------|------|
| `--no-verify` | 跳过本地 Git 钩子（`pre-commit`/`commit-msg` 等） |
| `--all` | 当暂存区为空时，自动 `git add -A` 将所有改动纳入本次提交 |
| `--amend` | 在不创建新提交的情况下**修补**上一次提交 |
| `--signoff` | 附加 `Signed-off-by` 行（遵循 DCO 流程时使用） |
| `--emoji` | 在提交信息中包含 emoji 前缀 |
| `--scope <scope>` | 指定提交作用域（如 `ui`、`docs`、`api`） |
| `--type <type>` | 强制提交类型（如 `feat`、`fix`、`docs` 等），覆盖自动判断 |

## What I won't do

- 调用任何包管理器/构建命令（无 `pnpm`/`npm`/`yarn` 等）
- 直接编辑工作区文件（只读写 `.git/COMMIT_EDITMSG` 与暂存区）
- 在 rebase/merge 冲突、detached HEAD 等状态下未经提示继续执行

## Type 与 Emoji 映射（使用 --emoji 时）

| Emoji | Type | 说明 |
|-------|------|------|
| ✨ | `feat` | 新增功能 |
| 🐛 | `fix` | 缺陷修复 |
| 📝 | `docs` | 文档与注释 |
| 🎨 | `style` | 风格/格式（不改语义） |
| ♻️ | `refactor` | 重构 |
| ⚡️ | `perf` | 性能优化 |
| ✅ | `test` | 新增/修复测试、快照 |
| 🔧 | `chore` | 构建/工具/杂务 |
| 👷 | `ci` | CI/CD 配置与脚本 |
| ⏪️ | `revert` | 回滚提交 |
| 💥 | `feat!` | 破坏性变更 |

> 若传入 `--type`/`--scope`，将**覆盖**自动推断
> 仅在指定 `--emoji` 标志时才会包含 emoji

## Best Practices for Commits

- **Atomic commits**：一次提交只做一件事，便于回溯与审阅
- **先分组再提交**：按目录/模块/功能点拆分
- **清晰主题**：首行 ≤ 72 字符，祈使语气
- **正文含上下文**：说明动机、方案、影响范围
- **遵循 Conventional Commits**：`<type>(<scope>): <subject>`

## Guidelines for Splitting Commits

1. **不同关注点**：互不相关的功能/模块改动应拆分
2. **不同类型**：不要将 `feat`、`fix`、`refactor` 混在同一提交
3. **文件模式**：源代码 vs 文档/测试/配置分组提交
4. **规模阈值**：超大 diff（>300 行或跨多个顶级目录）建议拆分
5. **可回滚性**：确保每个提交可独立回退

## Examples

### Detailed Commit Message Structure

完整的提交信息应包含以下部分：

```
<type>(<scope>): <subject>

- <body 描述动机和实现方式>
- <body 详细说明影响范围>
- <body 补充技术细节>

[可选 footer：BREAKING CHANGE、Closes、Refs 等]
```

**示例：**

```
feat(auth): add OAuth2 login flow with Google and GitHub providers

- 实现 Google OAuth2 第三方登录流程，包括授权码交换和用户信息获取
- 实现 GitHub OAuth2 第三方登录流程，支持 scopes 自定义
- 添加统一的登录回调处理逻辑，抽象 AuthCallbackHandler
- 将用户登录状态持久化到 localStorage，支持跨标签页同步

- 修复 token 刷新竞态条件，避免并发刷新请求
- 新增 isAuthenticated 和 currentUser 全局状态管理

Closes #42
Ref: #38, #45
```

---

### Commit Types with Detailed Examples

#### ✨ feat: 新增功能

```
feat(auth): add user authentication flow with JWT token support

- 实现基于 JWT 的用户认证流程，支持 token 自动刷新
- 新增 LoginForm 组件，包含邮箱/密码输入和记住我选项
- 添加登录失败重试机制，最大重试次数 3 次
- 集成第三方社交登录：Google、GitHub、Apple

- JWT payload 包含 userId、email、role、exp
- token 存储于 httpOnly cookie，提升安全性
- 登录成功后重定向到原始访问页面

BREAKING CHANGE: 认证 API 从 /api/login 迁移到 POST /api/auth/login
```

#### 🐛 fix: 缺陷修复

```
fix(api): resolve race condition in token refresh mechanism

- 修复多个并发请求同时触发 token 刷新导致的数据竞争问题
- 避免重复刷新请求，使用 Promise 缓存机制确保单次刷新
- 修复 refresh token 过期时无限重试的循环

- 问题根因：多个请求同时检测到 token 过期，各自发起刷新
- 影响范围：所有需要认证的 API 调用
- 测试覆盖：新增 tokenRefreshRaceCondition 测试用例

Fixes #156
```

#### 📝 docs: 文档更新

```
docs(api): update REST API documentation for v2 endpoints

- 更新所有 REST API 端点的请求/响应示例
- 添加 OpenAPI 3.0 规范文件，支持在线文档预览
- 补充认证流程说明和常见问题 FAQ
- 更新 API 变更日志，记录 v1 到 v2 的 breaking changes

- 新增认证与授权章节，详细说明 JWT 使用方式
- 添加错误码对照表，覆盖所有 4xx/5xx 响应
- 补充 rate limiting 说明，包括默认配额和升级方式

Related: #89
```

#### ♻️ refactor: 重构

```
refactor(core): extract shared retry logic into RetryableOperation helper

- 将分散在各个服务中的重试逻辑抽取为统一的 RetryableOperation
- 支持指数退避策略，最大重试次数和超时时间可配置
- 简化业务代码，重试逻辑与业务逻辑分离

- 原实现：在每个服务中重复实现相同重试逻辑，约 150 行重复代码
- 新实现：复用 RetryableOperation<T> 泛型类，配置驱动
- 影响范围：所有网络请求相关服务（UserService、PaymentService、NotificationService）

Performance: 降低平均响应延迟 15%，减少不必要重试 40%
```

#### ⚡️ perf: 性能优化

```
perf(database): optimize query performance for dashboard statistics

- 为 dashboard 常用查询添加复合索引，加速统计报表加载
- 实现查询结果缓存层，热门数据缓存时间 5 分钟
- 优化 N+1 查询问题，将关联查询合并为批量查询

- 原查询：dashboard 首页加载耗时 2.3s（12 次独立查询）
- 优化后：首页加载耗时 320ms（3 次批量查询 + 缓存）
- 内存占用：缓存层额外占用约 50MB，支持 1000 QPS

Benchmark: ab -n 1000 -c 10 → 从 45 req/s 提升至 180 req/s
```

#### ✅ test: 测试相关

```
test(auth): add comprehensive unit tests for JWT token validation

- 新增 token 签名验证、过期检查、payload 解析等单元测试
- 添加边界条件测试：空 token、篡改 token、过期 token
- 实现 MockAuthService 用于隔离测试依赖

- 测试覆盖率：从 45% 提升至 87%
- 新增测试用例：32 个通过，0 个失败
- 集成到 CI pipeline，每次 PR 自动执行

Co-authored-by: reviewer@example.com
```

#### 🔧 chore: 构建/工具/杂务

```
chore(deps): upgrade dependencies to latest stable versions

- 升级所有 npm 依赖到最新稳定版本
- 移除已废弃的包：deprecated-package
- 更新 TypeScript 到 5.2，启用更严格的类型检查

- 主要升级：React 18.2、Node.js 20.x、Eslint 8.56
- Breaking changes：已处理所有需要代码调整的变更
- 安全修复：修复 3 个中危漏洞，1 个高危漏洞

Audit: npm audit fix → 0 vulnerabilities
```

#### 👷 ci: CI/CD 配置

```
ci(github): optimize GitHub Actions workflow for faster feedback

- 优化 CI pipeline 执行顺序，先运行快速反馈的测试
- 新增并行执行：单元测试、类型检查、代码格式化检查
- 添加缓存策略：node_modules、build artifacts、cypress run-results

- 原完整 CI 时间：12 分钟 30 秒
- 优化后完整 CI 时间：4 分钟 15 秒
- 开发者反馈循环：从 13 分钟缩短至 5 分钟

Speed: Fast→ PR feedback 平均时间从 13min 降至 5min
```

---

### Special Cases

#### Revert 提交

```
⏪️ revert: revert "feat(payment): add Stripe payment integration"

- 回滚 Stripe 支付集成，因生产环境发现严重安全漏洞
- 回滚 commit：a1b2c3d4e5f6

This reverts the following commits:
  a1b2c3d4e5f6 feat(payment): add Stripe payment integration
  b2c3d4e5f6a78 fix(payment): fix webhook signature verification

Fixes #234
```

#### Breaking Change（破坏性变更）

```
💥 feat(auth)!: migrate from session-based to JWT authentication

- 完全迁移认证系统从 session 到 JWT
- 移除所有 session 相关代码和配置
- 更新所有 API 端点以接受 Authorization header

BREAKING CHANGE: 认证 API 已完全重新设计
- 旧端点：POST /api/login (cookie-based session)
- 新端点：POST /api/auth/login (JWT via Authorization header)
- 客户端需更新：存储 token 于 localStorage，添加 Authorization header

Migration guide: see docs/migration-v2.md
Related: #198, #201
```

#### 国际化提交信息

根据项目历史提交语言自动选择：

**中文项目：**
```
feat(用户模块): 新增用户注册功能，支持手机号和邮箱两种方式

- 实现手机号验证码发送和验证流程
- 实现邮箱链接激活账户流程
- 添加密码强度校验规则
- 集成短信和邮件服务商的统一接口

- 验证码有效期：5 分钟
- 密码要求：8位以上，包含大小写字母和数字
- 支持国际手机号格式验证

Closes #42
```

---

### Quick Reference

| Type | 当...时使用 | 中文说明 |
|------|------------|----------|
| `feat` | 新增功能、用户体验改进 | 新功能 |
| `fix` | 修复 bug、错误修正 | 缺陷修复 |
| `docs` | 文档、注释更新 | 文档 |
| `style` | 代码格式（不改语义） | 风格 |
| `refactor` | 重构（不改功能） | 重构 |
| `perf` | 性能优化 | 性能优化 |
| `test` | 测试相关 | 测试 |
| `chore` | 构建、工具、依赖 | 杂务 |
| `ci` | CI/CD 配置 | CI/CD |
| `revert` | 回滚提交 | 回滚 |

## Interaction style

- 清晰提示当前状态和可执行操作
- 必要时建议拆分提交并给出明确分组
- 尊重本地 Git 钩子配置
- 重要操作前进行确认（如 rebase/merge 冲突状态）

> 注：如框架不支持交互式确认，可在 front-matter 中开启 `confirm: true` 以避免误操作。
