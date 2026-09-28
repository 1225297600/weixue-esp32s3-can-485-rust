use defmt::{error, info};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use crate::app::routes::{api_status, api_time, index, api_time_set};
use picoserve::routing::{get, post, parse_path_segment};

pub const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn task_web(task_id: usize, stack: Stack<'static>) -> ! {
    info!("[web] task {} starting", task_id);

    if let Some(config) = stack.config_v4() {
        info!("[web] IP: {}", config.address);
    }

    let app = picoserve::Router::new()
        .route("/", get(index))
        .route("/api/status", get(api_status))
        .route("/api/time", get(api_time))   // ← 加这行
        .route(("/api/time/set", parse_path_segment::<u64>()), post(api_time_set));
    // 直接在每次任务运行时创建 Config，不再用 StaticCell
    let config = picoserve::Config::const_default().keep_connection_alive();

    let mut tcp_rx_buffer = [0u8; 1024];
    let mut tcp_tx_buffer = [0u8; 1024];
    let mut http_buffer = [0u8; 2048];

    info!("[web] listening on port 80...");

    picoserve::Server::new(&app, &config, &mut http_buffer)
        .listen_and_serve(task_id, stack, 80, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}


