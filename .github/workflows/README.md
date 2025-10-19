# GitHub Actions 自动发布说明

## 📋 配置说明

此工作流会在推送版本标签时自动触发，构建 Windows 版本的安装包并创建 GitHub Release。

## 🚀 使用方法

### 发布新版本

1. **更新版本号**
   - `package.json` 的 `version` 字段
   - `src-tauri/tauri.conf.json` 的 `version` 字段

2. **提交代码**
   ```bash
   git add .
   git commit -m "chore: bump version to 1.1.1"
   git push
   ```

3. **打标签并推送**（这会触发自动构建）
   ```bash
   git tag v1.1.1
   git push origin v1.1.1
   ```

4. **等待构建完成**
   - 访问 `https://github.com/你的用户名/quick-task/actions`
   - 查看构建进度（通常需要 5-15 分钟）
   - 构建成功后会自动创建 Release

5. **查看 Release**
   - 访问 `https://github.com/你的用户名/quick-task/releases`
   - 下载生成的安装包测试

## 📦 生成的文件

- `QuickTask_版本号_x64-setup.exe` - Windows 64位安装包

## ⚙️ 配置文件

- `release.yml` - 主要的工作流配置
  - 触发条件：推送 `v*` 标签
  - 运行环境：`windows-latest`
  - 自动创建 Release 并上传安装包

## 🔧 自定义

### 修改为草稿发布

如果希望手动检查后再发布，修改 `release.yml`：

```yaml
releaseDraft: true  # 改为 true
```

### 添加预发布版本

对于 beta 版本，可以这样：

```yaml
on:
  push:
    tags:
      - 'v*'
      - 'v*-beta*'  # 支持 v1.1.1-beta.1 格式
```

## ❓ 常见问题

### Q: 构建失败怎么办？
A: 访问 Actions 页面查看详细日志，常见原因：
- 版本号格式不正确
- 依赖安装失败
- 构建命令出错

### Q: 如何取消发布？
A: 可以在 Actions 页面手动取消正在运行的工作流

### Q: 如何删除错误的 Release？
A: 访问 Releases 页面，点击对应版本右侧的删除按钮

## 📝 标签命名规范

建议使用语义化版本号：
- `v1.0.0` - 主要版本
- `v1.1.0` - 次要版本（新功能）
- `v1.1.1` - 修订版本（Bug 修复）
- `v1.1.1-beta.1` - 预发布版本

