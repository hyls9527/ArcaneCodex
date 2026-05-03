# 贡献指南

感谢你的兴趣！这个项目是我为自己写的，但如果你觉得有用，欢迎一起完善。

## 怎么贡献

1. **Fork 仓库**
2. **创建分支** - `git checkout -b feature/你的功能`
3. **提交更改** - `git commit -m '添加某功能'`
4. **推送分支** - `git push origin feature/你的功能`
5. **创建 Pull Request**

## 开发环境

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

## 提交前检查

- [ ] 代码能编译 (`cargo check`)
- [ ] TypeScript 无错误 (`npx tsc --noEmit`)
- [ ] 测试通过 (`cargo test`)

## 报告 Bug

直接开 Issue，描述清楚：
- 发生了什么
- 怎么复现
- 期望发生什么
- 你的环境（Windows 版本、软件版本）

## 提功能建议

也开 Issue，说说：
- 你想要什么
- 为什么需要
- 有没有类似工具做过

## 代码风格

- Rust: 用 `cargo fmt` 格式化
- TypeScript: 用项目里的 ESLint 配置
- 注释用中文（这个项目是中文的）

## 许可证

MIT。你贡献的代码也会是 MIT 许可证。

---

有问题直接问，不用客气。
