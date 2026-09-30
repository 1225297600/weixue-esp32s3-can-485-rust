use core::cell::RefCell;
use core::fmt::Write as _;

use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use defmt::info;

pub static RS485_TX_CHANNEL: Channel<CriticalSectionRawMutex, heapless::Vec<u8, 256>, 4> =
    Channel::new();

pub static RS485_RX_BUF: Mutex<CriticalSectionRawMutex, RefCell<heapless::Vec<u8, 256>>> =
    Mutex::new(RefCell::new(heapless::Vec::new()));

fn hex_to_string<const N: usize>(data: &[u8]) -> heapless::String<N> {
    let mut s = heapless::String::<N>::new();
    for (i, b) in data.iter().enumerate() {
        if i > 0 {
            let _ = s.push(' ');
        }
        let _ = write!(&mut s, "{:02X}", b);
    }
    s
}

#[embassy_executor::task]
pub async fn task_rs485(
    mut uart: esp_hal::uart::Uart<'static, esp_hal::Async>,
    mut de: esp_hal::gpio::Output<'static>,
) {
    let mut rx_buf = [0u8; 256];

    loop {
        match select(
            RS485_TX_CHANNEL.receive(),          // 有数据要发
            uart.read_async(&mut rx_buf),        // 有数据可收
        )
        .await
        {
            // ---------- 发送分支 ----------
            Either::First(frame) => {
                info!("RS485 TX: {}", hex_to_string::<800>(&frame).as_str());

                let _ = de.set_high();
                Timer::after(Duration::from_micros(50)).await;

                let _ = uart.write_async(&frame).await;
                let _ = uart.flush_async().await;

                Timer::after(Duration::from_micros(50)).await;
                let _ = de.set_low();
            }

            // ---------- 接收分支 ----------
            Either::Second(Ok(0)) => {
                // 理论上不会走到，read_async 返回 0 一般是 EOF
            }
            Either::Second(Ok(n)) => {
                let mut v: Vec<u8, 256> = Vec::new();
                let _ = v.extend_from_slice(&rx_buf[..n]);

                info!("RS485 RX: {}", hex_to_string::<800>(&v).as_str());

                RS485_RX_BUF.lock(|cell| {
                    *cell.borrow_mut() = v;
                });
            }
            Either::Second(Err(_)) => {
                // UART 错误，忽略，继续
            }
        }
    }
}