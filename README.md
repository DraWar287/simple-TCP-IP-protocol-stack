# simple-TCP-IP-protocol-stack
A simple implementation of the TCP/IP protocol stack.
一个 TCP/IP 协议栈的简单实现，使用rust语言。
## 项目结构
```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── link
│   │   ├── ethernet.rs
│   │   └── mod.rs
│   ├── main.rs
│   └── net
│       ├── ipv4.rs
│       └── mod.rs
└── target
```
## 测试
* 单元测试
```bash
cargo test -- --nocapture
```



