# tessoa 插件模板

[English](README.md)

用 TypeScript 或 Rust 写 tessoa 插件的起点。插件往 tessoa 的右键菜单里加命令，运行在沙箱里：
它能看到用户选中的文件，但看不到路径，对文件的任何改动都只能请 tessoa 去做。

模板的布局让编程助手只凭一句话的需求就能做完整件事：`AGENTS.md` 把它引到 `sdk/` 里的说明书，
例子是完整可用的插件，三条命令负责测试、打包和装进 tessoa。

## 开始

1. 用这个模板建仓（GitHub 上的「Use this template」），克隆到本机。
2. 用你的编程助手打开它，说出想要的插件，比如「右键选中照片，按拍摄日期重命名」。

也可以自己动手：

```
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts   # Windows
bash scripts/setup.sh --lang ts                                    # macOS、Linux
tessoa-plugin init --lang ts --id com.example.my-plugin
tessoa-plugin test
tessoa-plugin package
tessoa-plugin install
```

装环境脚本只装缺的：TypeScript 要 Node.js，Rust 要 Rust，两者都要命令行工具 `tessoa-plugin`。
`init` 把所选语言的骨架挪到仓根、填好 id，并删掉另一种。`test` 在真正的沙箱里、对着模拟的 tessoa
运行插件，所以直到 `install` 之前都不需要 tessoa；`install` 把插件交给本机正在运行的 tessoa，
插件要的权限由你在 tessoa 窗口里确认。

## 目录

| 路径 | 是什么 |
|---|---|
| `AGENTS.md` | 编程助手的起点，指向 `sdk/AGENTS.md`。 |
| `skeleton/` | TypeScript 与 Rust 两套起点，`init` 之后就没了。 |
| `examples/` | 完整的插件，各自带测试。 |
| `sdk/` | SDK 套件：说明书、接口定义、接口参考、TypeScript 声明和 Rust SDK。不要改。 |
| `scripts/` | 装环境脚本。 |

仓根还有几个一行的文件，用的是某些工具会找的文件名，内容都是让它们去读 `AGENTS.md`。

## 需要什么

- Windows、macOS 或 Linux。
- 支持插件的 tessoa，用来装上并试用插件。装进 tessoa 需要授权；建仓、测试、打包都不需要。

## 升级 SDK

```
tessoa-plugin upgrade            # 加 --check 只查看不升级
```

有新版时先把 `tessoa-plugin` 自己换新，再把 `sdk/` 换成最新的套件。两者都从 download.tessoa.com 下载，换之前先核对签名和校验和；
`sdk/` 以外的代码不会被改动。升级后再跑一遍测试。

## 许可

模板里的骨架、例子和装环境脚本归你随便用。它们按下面的 MIT No Attribution 许可发布：用它们做出来的插件，
使用、修改、发布、出售都不用问任何人，也不用保留任何声明。

`sdk/` 下的文件是 tessoa 插件 SDK，不在这份许可之内，按 SDK 自己的条款。

```
MIT No Attribution

Copyright 2026 Chengdu Tianji Engineering Management Service Co., Ltd.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
