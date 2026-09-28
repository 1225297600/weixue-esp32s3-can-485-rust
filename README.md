# ESP32-S3-RS485-CAN Web 控制平台

基于 Rust + Embassy 异步框架开发的 ESP32-S3-RS485-CAN 工业通信控制平台。通过网页界面控制硬件、收发数据，支持 CAN、RS485、GPIO、RTC、BLE 等外设。

## 硬件平台

- **主控**：ESP32-S3-WROOM-1（Xtensa LX7 双核 @ 240MHz）
- **存储**：16MB Flash / 8MB PSRAM
- **无线**：2.4GHz Wi-Fi + Bluetooth 5 (LE)
- **接口**：隔离型 RS485 ×1、隔离型 CAN ×1、USB Type-C
- **供电**：DC 7~36V（接线端子）或 5V（USB Type-C）
- **参考文档**：
  - [Waveshare ESP32-S3-RS485-CAN 文档](https://docs.waveshare.net/ESP32-S3-RS485-CAN)
  - [原理图 PDF](https://www.waveshare.net/w/upload/a/a5/ESP32-S3-RS485-CAN-Schematic.pdf)

## 功能列表

### 已规划功能

| 功能 | 说明 | 状态 |
|------|------|------|
| **Wi-Fi** | STA 模式连接路由器，DHCP 获取 IP | ✅ 已实现 |
| **Web 服务器** | 基于 picoserve，提供 HTTP 接口 | ✅ 已实现 |
| **CAN 收发** | 通过网页发送/接收 CAN 帧，支持标准帧和扩展帧 | 🚧 开发中 |
| **RS485 收发** | 通过网页发送/接收 RS485 数据，半双工方向自动控制 | 🚧 开发中 |
| **GPIO 控制** | 网页控制用户 IO（GPIO1/GPIO2）电平，读取输入状态 | 🚧 开发中 |
| **RTC 时钟** | PCF85063 读写时间，网页显示和设置 | 🚧 开发中 |
| **BLE** | 蓝牙低功耗通信，可用于配网或数据传输 | 🚧 开发中 |

### 待定功能

- Modbus RTU / Modbus TCP 协议支持
- MQTT 上报与订阅
- OTA 在线升级
- 数据记录与导出
- 多设备组网

## 引脚分配与功能

| IO | TO | FUNC | CHIP | 说明 |
|----|------|------|------|------|
| 0 | BOOT | KEY | / | 启动模式选择按键，按下为下载模式 |
| 1~2 | / | EX-4P | / | 引出至 SH1.0 端子，可作 GPIO 使用 |
| 3~14 | / | Inside-20P | / | 内部 20P 排针，未引出至外部 |
| 15 | TX | TWAI-CAN | / | CAN 发送（TWAI TX） |
| 16 | RX | TWAI-CAN | / | CAN 接收（TWAI RX） |
| 17 | TX | UART-RS485 | / | RS485 发送数据 |
| 18 | RX | UART-RS485 | / | RS485 接收数据 |
| 19 | DN | USB | / | USB D-（内部 PHY） |
| 20 | DP | USB | / | USB D+（内部 PHY） |
| 21 | DE | UART-RS485 | / | RS485 半双工方向控制（高=发送，低=接收） |
| 38 | SCL | IIC-RTC | PCF85063 | RTC 时钟线 |
| 39 | SDA | IIC-RTC | PCF85063 | RTC 数据线 |
| 40 | INT | IIC-RTC | PCF85063 | RTC 中断输出，可配置为定时唤醒或事件通知 |
| U0RX | RX | Debug/Download | / | 串口下载/调试接收（GPIO44） |
| U0TX | TX | Debug/Download | / | 串口下载/调试发送（GPIO43） |

> **注意**：GPIO19/20 被 USB 占用；GPIO15~18、GPIO21 用于 CAN 和 RS485；GPIO38~40 用于 RTC；GPIO1~2 引出至 SH1.0；GPIO3~14 位于内部 20P 排针。

## 项目结构

```
wx_esp32s3/
├── Cargo.toml
├── .cargo/
│   └── config.toml              # 烧录配置
├── README.md
└── src/
    ├── bin/
    │   └── main.rs              # 入口：外设初始化 + 任务启动
    └── app/
        ├── mod.rs
        ├── task_net.rs          # Wi-Fi 连接 + DHCP + 网络栈
        ├── task_web.rs          # HTTP 服务器
        ├── routes.rs            # HTTP 路由
        ├── task_can.rs          # CAN 收发任务（待实现）
        ├── task_rs485.rs        # RS485 收发任务（待实现）
        ├── task_io.rs           # GPIO 控制任务（待实现）
        ├── task_rtc.rs          # RTC 时钟任务（待实现）
        └── task_ble.rs          # BLE 任务（待实现）
```

## 依赖

```toml
[dependencies]
esp-hal = { version = "~1.2.2", features = ["defmt", "esp32s3", "unstable"] }
esp-radio = { version = "1.0.0-beta.1", features = ["ble", "coex", "defmt", "esp-alloc", "esp32s3", "unstable", "wifi"] }
esp-rtos = { version = "0.4.0", features = ["defmt", "embassy", "esp-alloc", "esp-radio", "esp32s3"] }
esp-alloc = { version = "0.11.0", features = ["defmt", "esp32s3"] }
esp-println = { version = "0.18.0", features = ["defmt-espflash", "esp32s3"] }
esp-bootloader-esp-idf = { version = "0.6.0", features = ["defmt", "esp32s3"] }

embassy-executor = { version = "0.10.0", features = ["defmt"] }
embassy-time = { version = "0.5.0", features = ["defmt"] }
embassy-net = { version = "0.9.1", features = ["defmt", "dhcpv4", "medium-ethernet", "tcp", "udp"] }
embassy-sync = { version = "0.7", features = ["defmt"] }

picoserve = { version = "0.18", features = ["embassy"] }
static_cell = "2.1.1"

bt-hci = "0.9.0"
trouble-host = { version = "0.7.0", features = ["gatt"] }

embedded-can = "0.4"
embedded-io = { version = "0.7.1", features = ["defmt"] }
embedded-io-async = { version = "0.7.0", features = ["defmt"] }

pcf85063a = "0.1.1"
time = { version = "0.3", default-features = false }
```

## 构建与烧录

### 环境准备

```bash
# 安装 espflash
cargo install espflash

# 安装 Xtensa 目标
rustup target add xtensa-esp32s3-none-elf
```

### 配置烧录串口

在 `.cargo/config.toml` 中指定串口：

```toml
[target.'cfg(all(target_arch = "xtensa", target_os = "none"))']
runner = "espflash flash --monitor --port /dev/ttyACM0"
```

### 编译运行

```bash
cargo run
```

### 指定串口（临时）

```bash
cargo run -- --port /dev/ttyUSB0
```

## 使用说明

### 1. 配置 Wi-Fi

编辑 `src/app/task_net.rs`：

```rust
const WIFI_NAME: &str = "你的WiFi名称";
const WIFI_PASSWORD: &str = "你的WiFi密码";
```

### 2. 获取设备 IP

烧录后串口会打印分配的 IP 地址：

```
[INFO ] [net] IPv4 address: 192.168.20.230/23
```

### 3. 访问 Web 界面

在同一局域网内的浏览器访问：

```
http://192.168.20.230/
```

### 4. Web API

| 路径 | 方法 | 说明 |
|------|------|------|
| `/` | GET | 控制面板首页 |
| `/api/status` | GET | 设备状态 JSON |
| `/api/can/send` | POST | 发送 CAN 帧（待实现） |
| `/api/can/recv` | GET | 获取接收到的 CAN 帧（待实现） |
| `/api/rs485/send` | POST | 发送 RS485 数据（待实现） |
| `/api/rs485/recv` | GET | 获取 RS485 接收数据（待实现） |
| `/api/io/set` | POST | 设置 GPIO 电平（待实现） |
| `/api/io/get` | GET | 读取 GPIO 状态（待实现） |
| `/api/rtc/get` | GET | 读取 RTC 时间（待实现） |
| `/api/rtc/set` | POST | 设置 RTC 时间（待实现） |
| `/api/ble/status` | GET | BLE 状态（待实现） |

## 硬件注意事项

1. **优化等级**：`esp-radio` 的 Wi-Fi/BLE 驱动需要优化等级 2 或 3，`Cargo.toml` 中已配置：
   ```toml
   [profile.dev.package.esp-radio]
   opt-level = 3
   ```

2. **RS485 方向控制**：GPIO21 为半双工方向控制引脚（DE），发送时拉高，接收时拉低。推荐使用 UART RTS 硬件流控自动切换。

3. **终端电阻**：RS485 和 CAN 各预留 120Ω 匹配电阻，通过跳线帽使能。多设备总线时确保两端设备启用。

4. **供电**：接线端子 DC 7~36V，USB Type-C 5V。工业场景建议使用接线端子。

5. **USB 引脚**：GPIO19/20 被 USB 占用，RS485 和 CAN 已避开，不冲突。

6. **RTC 电池**：PCF85063 需接入可充电 RTC 电池（SH1.0 接口）才能断电保持时间。INT 引脚（GPIO40）可配置为中断输出，用于定时唤醒或事件通知。

7. **TWAI 已知问题**：部分 `esp-hal` 版本 TWAI 接收可能阻塞，建议关注 [esp-rs/esp-hal#2281](https://github.com/esp-rs/esp-hal/issues/2281)。

## 开发状态

- ✅ Wi-Fi STA 连接
- ✅ DHCP 获取 IP
- ✅ HTTP 服务器基础框架
- 🚧 CAN 收发网页控制
- 🚧 RS485 收发网页控制
- 🚧 GPIO 网页控制
- 🚧 RTC 网页读写
- 🚧 BLE 通信

## 许可证

MIT OR Apache-2.0
