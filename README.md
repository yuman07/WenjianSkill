# 问剑长生 · 神通升级规划工具

> 手游《问剑长生》的神通升级规划桌面应用，帮助玩家计算最优的按周升级路径。

## 功能特性

- **一键规划** — 输入 6 个战斗神通的当前状态（境界、职业、商店、等级、剩余书页）和目标等级，自动计算最少周数达成所有目标
- **逐周操作指引** — 生成每周详细步骤：兑换书页、转换书页、升级神通，按图索骥不再迷路
- **穷举最优解** — 达成目标后自动利用剩余资源继续提升等级，穷举搜索保证方案最优
- **导出方案** — 支持导出规划方案为 TXT 文件，方便分享或离线查阅
- **自动保存** — 所有设置自动持久化，重启应用后保留上次输入

## 截图

<table>
  <tr>
    <td><img src="Screenshots/a.png" alt="神通配置" /></td>
    <td><img src="Screenshots/b.png" alt="材料设置" /></td>
  </tr>
  <tr>
    <td><img src="Screenshots/c.png" alt="规划结果" /></td>
    <td><img src="Screenshots/d.png" alt="逐周方案" /></td>
  </tr>
</table>

## 安装

前往 [Releases](https://github.com/yuman07/WenjianSkill/releases/latest) 页面下载对应平台的安装包。

### macOS

1. 下载 `WenjianSkill_x.x.x_aarch64.dmg`
2. 打开 DMG 文件，将应用拖入「应用程序」文件夹
3. 首次打开如提示"无法验证开发者"，前往 **系统设置 → 隐私与安全性**，点击「仍要打开」即可

> 仅支持 Apple Silicon (M 系列芯片) Mac。

### Windows

1. 下载 `wenjian-skill.exe`
2. 双击即可运行，无需安装

> 便携式应用，可放在任意目录。仅支持 64 位 Windows 10 及以上。

## 算法说明

规划引擎分三个阶段运行：

1. **二分搜索最少周数** — 可行性检查验证资源总量的充分必要条件，二分搜索给出精确最优解
2. **穷举搜索 bonus 等级** — 遍历所有可能的额外等级组合（最多 15,625 种），找到总等级提升最大的可行方案
3. **逐周模拟生成操作步骤** — 按优先级执行转换和升级，低珍贵度资源优先消耗

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 (macOS / Windows) |
| 前端 | React 19 + TypeScript + Tailwind CSS 4 |
| 后端算法 | Rust |
| 构建工具 | Vite |

## 开发

```bash
# 安装依赖
devbox run -- npm install

# 开发模式
devbox run -- npm run tauri dev

# 构建
devbox run -- npm run tauri build
```

## License

MIT
