// src/app/task_web.rs
use picoserve::response::{Response, StatusCode};

use crate::app::task_rtc::RTC_TIME_CHANNEL;
use core::fmt::{self, Write};

use picoserve::request::Path;

// ============================================================
// 首页
// ============================================================

/// 首页处理函数：返回简单的 HTML（含 RTC 时间显示）
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
    <h1>Hello from ESP32-S3!</h1>
    <p>Rust + embassy-net + picoserve</p>

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

    <script>
    async function setTime() {
        const dtStr = document.getElementById('dt-input').value;
        if (!dtStr) {
            document.getElementById('set-result').textContent = '请先选择时间';
            return;
        }
        // datetime-local 是本地时区，转成 Unix 秒
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
    pollTime();
    setInterval(pollTime, 1000);
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

    // heapless 0.8 的 String，owned 数据，能 move 进 Response
    let mut s = heapless::String::<40>::new();
    let _ = write!(&mut s, r#"{{"timestamp":{}}}"#, ts);

    Response::new(StatusCode::OK, s)   // ← 直接把 s 交出去，不要 .as_str()
        .with_header("Content-Type", "application/json")
}



// ============================================================
// 栈上写入工具
// ============================================================

/// 极简的栈上写入器：往 &mut [u8] 里写 str，
/// 用于拼装固定长度的小 JSON，避免引入 heapless 依赖。
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

