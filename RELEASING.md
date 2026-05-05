# FocusLab 发版流程

> 一次 `git tag + push` → 三平台安装包自动出炉到 GitHub Releases。

## 🚀 标准发版步骤

### 1. 确保代码已提交并推送

```bash
git status                     # 确保 working tree 干净
git push origin main           # 把最新 main 推上去
```

### 2. 打 tag 并推送

```bash
# 版本号语义遵循 SemVer：v主版本.次版本.补丁版本
git tag -a v0.1.0 -m "v0.1.0 首个内测版"
git push origin v0.1.0
```

### 3. 等 GitHub Actions 跑完（约 15-20 分钟）

打开仓库的 **Actions** 标签页，看 `Release` 工作流跑：

- ✅ Windows (x64)
- ✅ macOS Apple Silicon (aarch64)
- ✅ macOS Intel (x86_64)
- ✅ Linux (x64)

四个 job 并行，最慢的一台决定总时长。

### 4. 编辑 Release 草稿并发布

去 **Releases** 页面会看到一个标题为 `FocusLab v0.1.0` 的 **草稿**，里面已经挂好所有平台的安装包。

1. 点 **Edit** 修改 release notes（默认模板有占位）
2. 检查附件齐全：
   - `FocusLab_0.1.0_x64-setup.exe`（Windows NSIS，**最推荐给 Win 用户**）
   - `FocusLab_0.1.0_x64_en-US.msi`（Windows MSI）
   - `FocusLab_0.1.0_aarch64.dmg`（**Mac M 系列芯片用这个**）
   - `FocusLab_0.1.0_x64.dmg`（Mac Intel 老机型）
   - `focuslab_0.1.0_amd64.AppImage`（Linux 通用）
   - `focuslab_0.1.0_amd64.deb`（Debian / Ubuntu）
3. 点 **Publish release** 公开

---

## 🍎 Mac 用户首次安装提示

由于未做 Apple 代码签名 + 公证，Mac 用户首次打开会被 Gatekeeper 拦截。需要在 README 或 Release notes 里告知：

> **macOS 安装提示**：双击 `.dmg` 后把 FocusLab 拖到 Applications。
> 首次启动若提示"无法打开，因为无法验证开发者"，请：
>
> 1. **右键**应用图标 → **打开** → 在弹窗点 **打开**
> 2. 或在「**系统设置** → **隐私与安全性**」往下拉，点 **"仍要打开"**

如果未来想做无弹窗体验，需 Apple Developer ID（$99/年）+ 在 GitHub Secrets 配置签名/公证证书。

---

## 🧪 仅测试构建（不发版）

不打 tag 也能验证三平台能否构建成功：

- 推到 `main` 分支 → CI workflow 自动跑（只验证类型检查 + 单测，不出包）
- 想拿到包试装 → 去 Actions 页面手动触发 `Release` workflow（需要先打个测试 tag 如 `v0.0.0-test`）

---

## ⚠️ 常见坑

**Q: tag 推上去了但 Action 没跑？**
A: 检查 tag 名是不是以 `v` 开头（`release.yml` 只匹配 `v*`）。

**Q: Mac 包能在 Win 上打吗？**
A: 不能。Tauri 的 `.dmg` 必须在 macOS runner 上构建。这正是用 GitHub Actions 的核心价值——它提供真实的 Mac 虚拟机。

**Q: Action 跑失败了怎么办？**
A: 进入失败的 job 看日志。最常见原因：
- 前端类型错误 → 本地先跑 `pnpm exec vue-tsc --noEmit` 修干净
- Rust 编译错误 → 本地 `cargo check --manifest-path src-tauri/Cargo.toml`
- pnpm-lock.yaml 与 package.json 不同步 → 重跑 `pnpm install` 后提交

**Q: 想撤销一次 release？**
A:
```bash
# 删远程 tag
git push --delete origin v0.1.0
# 删本地 tag
git tag -d v0.1.0
# 然后去 GitHub Releases 页面手动删 release
```

---

## 🔐 未来：加签名

待商业化阶段做：

| 平台 | 需要 | 配置位置 |
|---|---|---|
| Windows | EV 代码签名证书 | repo Secrets：`WINDOWS_CERTIFICATE` + `WINDOWS_CERTIFICATE_PASSWORD` |
| macOS | Apple Developer ID + App-specific password | repo Secrets：`APPLE_CERTIFICATE` / `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` |

参考：https://v2.tauri.app/distribute/sign/
