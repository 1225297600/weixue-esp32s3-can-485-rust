// src/app/task_web.rs
use picoserve::response::{Response, StatusCode};

use core::fmt::{self, Write};

use crate::app::task_rtc::RTC_TIME_CHANNEL;
use crate::app::task_rs485::{RS485_TX_CHANNEL, RS485_RX_BUF};

/// 首页处理函数：返回简单的 HTML（含 RTC 时间显示 + RS485 收发）
pub async fn index() -> impl picoserve::response::IntoResponse {
    Response::new(
        StatusCode::OK,
        r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ESP32-S3 Web</title>
</head>
<body>
    <h1>Weixue ESP32-S3-CAN-485 Control</h1>

    <hr>

    <h2>RTC 时间</h2>
    <p>
        Unix 时间戳: <span id="rtc-ts">--</span><br>
        本地时间: <span id="rtc-local">--</span><br>
        UTC 时间: <span id="rtc-utc">--</span>
    </p>

    <h2>设置时间</h2>
    <p>
        <input type="datetime-local" id="dt-input" step="1">
        <button onclick="setTime()">用选中的时间设置</button>
        <button onclick="setNow()">用当前时间设置</button>
    </p>
    <p id="set-result"></p>

    <hr>

    <h2>RS485 收发</h2>
    <p>
        <input type="text" id="rs485-tx" size="40"
               placeholder="十六进制，如 01 03 00 00 00 0A C5 CD">
        <button onclick="rs485Send()">发送</button>
        <button onclick="rs485Clear()">清空接收</button>
    </p>
    <p id="rs485-result"></p>
    <p>最近接收帧:
        <code id="rs485-rx-hex">(空)</code><br>
        长度: <span id="rs485-rx-len">0</span> 字节
    </p>

    <hr>

    <script>
    async function setTime() {
        const dtStr = document.getElementById('dt-input').value;
        if (!dtStr) {
            document.getElementById('set-result').textContent = '请先选择时间';
            return;
        }
        const ts = Math.floor(new Date(dtStr).getTime() / 1000);
        await sendSetTime(ts);
    }

    async function setNow() {
        const ts = Math.floor(Date.now() / 1000);
        await sendSetTime(ts);
    }

    async function sendSetTime(ts) {
        try {
            const r = await fetch('/api/time/set/' + ts, { method: 'POST' });
            const j = await r.json();
            document.getElementById('set-result').textContent =
                '设置结果: ' + (j.status || r.status);
        } catch (e) {
            document.getElementById('set-result').textContent = 'ERR: ' + e.message;
        }
    }

    async function pollTime() {
        try {
            const r = await fetch('/api/time');
            if (!r.ok) throw new Error('HTTP ' + r.status);
            const j = await r.json();

            document.getElementById('rtc-ts').textContent = j.timestamp;

            const d = new Date(j.timestamp * 1000);
            document.getElementById('rtc-local').textContent =
                d.toLocaleString('zh-CN', { hour12: false });
            document.getElementById('rtc-utc').textContent =
                d.toISOString().replace('T', ' ').slice(0, 19) + ' UTC';
        } catch (e) {
            document.getElementById('rtc-local').textContent = 'ERR: ' + e.message;
        }
    }

    // ---------------- RS485 ----------------

    async function rs485Send() {
        let hex = document.getElementById('rs485-tx').value.replace(/[\s:,-]/g, '');
        if (!hex || hex.length % 2 !== 0 || /[^0-9a-fA-F]/.test(hex)) {
            document.getElementById('rs485-result').textContent =
                '请输入偶数长度十六进制字符串';
            return;
        }
        try {
            const r = await fetch('/api/rs485/send/' + hex, { method: 'POST' });
            const j = await r.json();
            document.getElementById('rs485-result').textContent =
                '发送: ' + (j.status || r.status);
        } catch (e) {
            document.getElementById('rs485-result').textContent = 'ERR: ' + e.message;
        }
    }

    async function rs485Clear() {
        try {
            await fetch('/api/rs485/clear', { method: 'POST' });
            document.getElementById('rs485-rx-hex').textContent = '(空)';
            document.getElementById('rs485-rx-len').textContent = '0';
        } catch (e) {}
    }

    async function pollRs485() {
        try {
            const r = await fetch('/api/rs485/recv');
            if (!r.ok) return;
            const j = await r.json();
            document.getElementById('rs485-rx-hex').textContent = j.hex || '(空)';
            document.getElementById('rs485-rx-len').textContent = j.len;
        } catch (e) {}
    }

    pollTime();
    setInterval(pollTime, 1000);
    pollRs485();
    setInterval(pollRs485, 1000);
    </script>
</body>
</html>
        "#,
    )
    .with_header("Content-Type", "text/html; charset=utf-8")
}

// ============================================================
// JSON API
// ============================================================

/// GET /api/status —— 返回设备状态
pub async fn api_status() -> impl picoserve::response::IntoResponse {
    Response::new(
        StatusCode::OK,
        r#"{"status":"ok","device":"esp32s3"}"#,
    )
    .with_header("Content-Type", "application/json")
}

/// GET /api/time —— 返回 {"timestamp": 1700000000}
pub async fn api_time() -> impl picoserve::response::IntoResponse {
    let ts = RTC_TIME_CHANNEL.try_receive().unwrap_or(0);

    let mut s = heapless::String::<40>::new();
    let _ = write!(&mut s, r#"{{"timestamp":{}}}"#, ts);

    Response::new(StatusCode::OK, s)
        .with_header("Content-Type", "application/json")
}

/// POST /api/time/set/:ts —— 用 Unix 秒设置 RTC
pub async fn api_time_set(ts: u64) -> impl picoserve::response::IntoResponse {
    use crate::app::task_rtc::SET_TIME_CHANNEL;

    match SET_TIME_CHANNEL.try_send(ts) {
        Ok(_) => Response::new(StatusCode::OK, r#"{"status":"queued"}"#)
            .with_header("Content-Type", "application/json"),
        Err(_) => Response::new(StatusCode::SERVICE_UNAVAILABLE, r#"{"status":"busy"}"#)
            .with_header("Content-Type", "application/json"),
    }
}

// ------------------------------------------------------------
// RS485 API
// ------------------------------------------------------------

/// 把 "01030000..." 解析成字节数组
fn parse_hex<const N: usize>(hex: &str) -> Option<heapless::Vec<u8, N>> {
    let bytes = hex.as_bytes();
    if bytes.is_empty() || bytes.len() % 2 != 0 {
        return None;
    }
    let mut out: heapless::Vec<u8, N> = heapless::Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let hi = (bytes[i] as char).to_digit(16)? as u8;
        let lo = (bytes[i + 1] as char).to_digit(16)? as u8;
        out.push((hi << 4) | lo).ok()?;
        i += 2;
    }
    Some(out)
}

/// POST /api/rs485/send/:hex —— 发送十六进制数据到 485 总线
pub async fn api_rs485_send(hex: heapless::String<512>) -> impl picoserve::response::IntoResponse {
    let data: heapless::Vec<u8, 256> = match parse_hex(hex.as_str()) {
        Some(v) => v,
        None => {
            return Response::new(StatusCode::BAD_REQUEST, r#"{"status":"bad hex"}"#)
                .with_header("Content-Type", "application/json");
        }
    };

    match RS485_TX_CHANNEL.try_send(data) {
        Ok(_) => Response::new(StatusCode::OK, r#"{"status":"queued"}"#)
            .with_header("Content-Type", "application/json"),
        Err(_) => Response::new(StatusCode::SERVICE_UNAVAILABLE, r#"{"status":"busy"}"#)
            .with_header("Content-Type", "application/json"),
    }
}

/// GET /api/rs485/recv —— 返回最近一帧 {"len":N,"hex":"AABB..."}
pub async fn api_rs485_recv() -> impl picoserve::response::IntoResponse {
    // String 容量：512 (hex) + 键名 / 标点 + 余量
    let s: heapless::String<640> = RS485_RX_BUF.lock(|cell| {
        let buf = cell.borrow();
        let mut s = heapless::String::<640>::new();
        let _ = write!(&mut s, r#"{{"len":{},"hex":""#, buf.len());
        for b in buf.iter() {
            let _ = write!(&mut s, "{:02X}", b);
        }
        let _ = s.push_str(r#""}"#);
        s
    });

    Response::new(StatusCode::OK, s)
        .with_header("Content-Type", "application/json")
}

/// POST /api/rs485/clear —— 清空接收缓冲
pub async fn api_rs485_clear() -> impl picoserve::response::IntoResponse {
    RS485_RX_BUF.lock(|cell| cell.borrow_mut().clear());
    Response::new(StatusCode::OK, r#"{"status":"cleared"}"#)
        .with_header("Content-Type", "application/json")
}

// ============================================================
// 栈上写入工具（保留备用）
// ============================================================

struct FixedBuf<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FixedBuf<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<'a> fmt::Write for FixedBuf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let b = s.as_bytes();
        if self.len + b.len() > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.len..self.len + b.len()].copy_from_slice(b);
        self.len += b.len();
        Ok(())
    }
}