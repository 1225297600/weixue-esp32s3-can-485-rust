// src/app/task_web.rs
use defmt::info;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use picoserve::routing::{get, post, parse_path_segment};

use crate::app::routes::{
    api_status,
    api_time,
    api_time_set,
    api_rs485_send,
    api_rs485_recv,
    api_rs485_clear,
    index,
};

pub const WEB_TASK_POOL_SIZE: usize = 4;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn task_web(task_id: usize, stack: Stack<'static>) -> ! {
    info!("[web] task {} starting", task_id);

    if let Some(config) = stack.config_v4() {
        info!("[web] IP: {}", config.address);
    }

    let app = picoserve::Router::new()
        .route("/",                 get(index))
        .route("/api/status",       get(api_status))
        .route("/api/time",         get(api_time))
        .route(
            ("/api/time/set", parse_path_segment::<u64>()),
            post(api_time_set),
        )
        .route(
            ("/api/rs485/send", parse_path_segment::<heapless::String<512>>()),
            post(api_rs485_send),
        )
        .route("/api/rs485/recv",   get(api_rs485_recv))
        .route("/api/rs485/clear",  post(api_rs485_clear));

    let config = picoserve::Config::const_default().keep_connection_alive();

    let mut tcp_rx_buffer = [0u8; 2048];
    let mut tcp_tx_buffer = [0u8; 4096];
    let mut http_buffer   = [0u8; 4096];

    info!("[web] listening on port 80...");

    picoserve::Server::new(&app, &config, &mut http_buffer)
        .listen_and_serve(task_id, stack, 80, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}