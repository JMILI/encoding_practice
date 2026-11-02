# encoding_practice

UDP 二进制协议相关的学习与实践，包含需求文档和交互式编解码教程。

## 子项目

| 目录 | 说明 |
|------|------|
| [protocol_tutorial](./protocol_tutorial/) | Web 应用（Axum）：协议组包/解包、UDP 仿真、位流可视化 |
| [docs/tet.html](./docs/tet.html) | **位域静态图解教程**（Canvas + C++ 结构体，浏览器直接打开） |
| [docs/需求.md](./docs/需求.md) | 协议字段定义、Hot 100 题目列表等原始需求 |

## 协议概要

UDP 单播消息，网络字节序，结构为：

```text
type (1B) + len (2B) + fc (变长) + data1 (2B) + data2 (4B)
```

- `fc`：从左到右编码，每字节最低位为续位标志
- `data1` / `data2`：位域结构，详见 `docs/需求.md`

## 教程两种打开方式

| 类型 | 地址 / 命令 | 说明 |
|------|-------------|------|
| **交互实验台** | `cargo run --bin web` → http://127.0.0.1:3000/ | 改字段、组包、模拟发送 |
| **位域图解教程** | 同上服务 → http://127.0.0.1:3000/guide | Canvas 位域图 + C++ 代码 |

也可单独用浏览器打开静态文件 [docs/tet.html](./docs/tet.html)（与 `/guide` 同源，但顶栏链接需通过 Web 服务访问才有效）。

## 快速开始（交互实验台）

```bash
# 从 deployProject 根目录
cd encoding_practice/protocol_tutorial
cargo run --bin web
```

浏览器打开（顶栏与「页面路由」区块均可点击跳转）：
- 实验台 **http://127.0.0.1:3000/**（别名 `/lab`）
- 图解教程 **http://127.0.0.1:3000/guide**
首次编译较慢（Axum + 协议库）。启动后可在界面中：

- 编辑协议字段并组包
- 查看十六进制、位流与内存布局
- 模拟 UDP 一次发送，或按位单步发送并解析

## 与 leetcode 项目的关系

`docs/需求.md` 中的 LeetCode Hot 100 列表是当年生成题目数据用的原始需求（`leetcode_hot100/`、`leetcode_acm/` 两个仓库不在本部署目录里，也不参与构建与运行）。阅读时把它当作“需求原文/历史记录”即可，不是部署说明。

## 文档

- 协议与 C++ 参考实现：[docs/需求.md](./docs/需求.md)
- 教程应用说明：[protocol_tutorial/README.md](./protocol_tutorial/README.md)
