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

**Good (使用 --emoji)**

```
✨ feat(ui): add user authentication flow
🐛 fix(api): handle token refresh race condition
📝 docs: update API usage examples
♻️ refactor(core): extract retry logic into helper
✅ test: add unit tests for rate limiter
🔧 chore: update git hooks and repository settings
⏪️ revert: revert "feat(core): introduce streaming API"
```

**Good (不使用 --emoji)**

```
feat(ui): add user authentication flow
fix(api): handle token refresh race condition
docs: update API usage examples
refactor(core): extract retry logic into helper
test: add unit tests for rate limiter
chore: update git hooks and repository settings
revert: revert "feat(core): introduce streaming API"
```

**Good (包含 Body)**

```
feat(auth): add OAuth2 login flow

- implement Google and GitHub third-party login
- add user authorization callback handling
- improve login state persistence logic

Closes #42
```

**Good (包含 BREAKING CHANGE)**

```
feat(api)!: redesign authentication API

- migrate from session-based to JWT authentication
- update all endpoint signatures
- remove deprecated login methods

BREAKING CHANGE: authentication API has been completely redesigned
```

## Interaction style

- 清晰提示当前状态和可执行操作
- 必要时建议拆分提交并给出明确分组
- 尊重本地 Git 钩子配置
- 重要操作前进行确认（如 rebase/merge 冲突状态）

> 注：如框架不支持交互式确认，可在 front-matter 中开启 `confirm: true` 以避免误操作。
