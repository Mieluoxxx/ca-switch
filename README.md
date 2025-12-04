# SiriusX Coding Agent Switch

多 AI 编程助手配置管理工具

## 支持

- Claude Code
- Codex
- Gemini CLI
- OpenCode

## 安装

```bash
cargo install --path .
```

## 使用

```bash
# 交互式菜单
ca-switch

# 配置管理
ca-switch api      # Claude
ca-switch codex    # Codex
ca-switch gemini   # Gemini
ca-switch opencode # OpenCode

# 其他
ca-switch backup   # 备份恢复
ca-switch status   # 查看状态
```

## 功能

- 🔄 快速切换配置
- 💾 WebDAV 云同步
- 🎨 交互式界面

## License

MIT
