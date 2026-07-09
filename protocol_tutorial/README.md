# protocol_tutorial

基于 Web 的 UDP 协议编解码交互式教程（Axum + 静态前端）。

## 功能

- 可视化编辑 `type`、`fc`、`data1`、`data2` 等协议字段
- 实时组包，展示十六进制、位描述与结构对照
- 模拟 UDP 环回收发（服务端逻辑在 Rust 后端完成，无需真实 UDP 端口）
- 支持随机填充、逐位发送、解析验证

## 环境要求

- Rust 2021
- 仅需标准 Rust 工具链，**无需** GTK / WebView / Dioxus Desktop 依赖

## 常用命令

```bash
cd encoding_practice/protocol_tutorial

cargo run --bin web          # 实验台 http://127.0.0.1:3000/  图解 http://127.0.0.1:3000/guide
PORT=3001 cargo run --bin web  # 指定端口
cargo build --release        # 发布构建
cargo test                   # 运行协议单元测试
```

## 端口说明

Web 版在浏览器中操作，不再绑定真实 UDP 端口。原桌面版的逻辑对应关系：

| 原桌面版 | Web 版 |
|----------|--------|
| `127.0.0.1:44001` 服务端发送 | `POST /api/simulate-full` 模拟一次发送 |
| `127.0.0.1:44002` 客户端接收 | 后端解析后返回客户端视图 |
| 按位 UDP 发送 | `准备按位发送` + `单步发送 1 位` + `完成按位帧并解析` |

## API

| 路径 | 说明 |
|------|------|
| `GET /` | 交互实验台 |
| `GET /guide` | 位域图解教程（`docs/tet.html`，Canvas + C++） |
| `POST /api/build` | 根据表单生成报文 |
| `POST /api/random` | 随机测试数据 |
| `POST /api/simulate-full` | 模拟 UDP 一次发送并解析 |
| `POST /api/prepare-bits` | 准备按位发送模式 |
| `POST /api/step-preview` | 按位发送单步预览 |
| `POST /api/finalize-bits` | 完成按位帧并解析 |

## 协议参考

字段布局与位域定义见上级目录 [docs/需求.md](../docs/需求.md)。

## 依赖

- `axum` — HTTP 服务
- `tokio` — 异步运行时
- `serde` / `serde_json` — API 序列化
- `rand` — 随机测试数据
