# ca-switch - OpenCode 配置管理工具

OpenCode 配置管理工具，支持多 Provider 和 Model 管理

## 支持

- OpenCode

## 安装

```bash
cargo install ca-switch
```

## 使用

```bash
# TUI 交互式界面（默认）
ca-switch

# 其他
ca-switch backup   # 备份恢复
ca-switch status   # 查看状态
```

## 功能

- 🔄 多 Provider 配置管理
- 🤖 模型列表管理与站点检测
- 💾 WebDAV 云同步备份
- 🎨 现代化 TUI 交互界面（三栏布局）

## TUI 界面

### 三栏布局

```
┌──────────┬─────────────┬───────────────────────────┐
│ Provider │   Models    │         详情面板          │
│  (25%)   │   (30%)     │         (45%)             │
└──────────┴─────────────┴───────────────────────────┘
```

### 快捷键

| 按键 | 功能 |
|------|------|
| `h` / `l` | 切换面板焦点 (Provider ↔ Model) |
| `j` / `k` | 上下导航 |
| `a` | 添加 Provider / Model（根据焦点） |
| `e` | 编辑 Provider |
| `d` | 删除 Provider / Model（根据焦点） |
| `Enter` | 进入多选应用模式 |
| `/` | 搜索模型 |
| `t` | 获取站点可用模型 |
| `Tab` | 切换 Tab 页 |
| `?` | 显示帮助 |
| `q` | 退出 |

## License

MIT
