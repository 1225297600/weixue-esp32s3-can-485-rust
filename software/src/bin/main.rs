#![allow(linker_messages)]
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use bt_hci::controller::ExternalController;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;

use esp_hal::i2c::master::Config as I2cConfig;
use esp_hal::i2c::master::I2c;

use esp_hal::twai::{BaudRate, EspTwaiFrame, StandardId, TwaiConfiguration, TwaiMode};
use embedded_can::Frame;

use wx_esp32s3::app::task_net::task_net;
use wx_esp32s3::app::task_rtc::task_rtc;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // ---- HAL 初始化 ----
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // ---- 初始化堆分配器 ----
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    esp_alloc::heap_allocator!(size: 64 * 1024);

    // ---- 初始化 RTOS ----
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    // ---- 初始化 BLE ----
    let transport = BleConnector::new(peripherals.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 1>::new(transport);
    let mut resources: HostResources<_, DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let _stack = trouble_host::new(ble_controller, &mut resources).build();

    // ---- 初始化 I2C ----
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO39)
        .with_scl(peripherals.GPIO38)
        .into_async();

    // ---- 初始化 TWAI (CAN) ----
    let twai_config = TwaiConfiguration::new(
        peripherals.TWAI0,
        peripherals.GPIO16,
        peripherals.GPIO15,
        BaudRate::B500K,
        TwaiMode::Normal,
    );
    let twai = twai_config.into_async().start();

    // ---- 启动网络任务 ----
    // task_net 内部完成 Wi-Fi 初始化 + 连接 + DHCP，
    // 并自己 spawn task_web，把 Stack 传过去。
    spawner.spawn(task_net(spawner, peripherals.WIFI).unwrap());

    // 其他任务（按需启用）
    spawner.spawn(task_rtc(i2c).unwrap());
    // spawner.spawn(task_can(twai).unwrap());
    // spawner.spawn(task_rs485().unwrap());
    // spawner.spawn(task_ble().unwrap());

    let _ = i2c;
    let _ = twai;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn task_can(mut twai: esp_hal::twai::Twai<'static, esp_hal::Async>) -> ! {
    let tx_frame = EspTwaiFrame::new(
        StandardId::new(0x123).unwrap(),
        &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    )
    .unwrap();

    loop {
        info!("can loop");

        match twai.transmit_async(&tx_frame).await {
            Ok(_) => info!("CAN TX ok, id=0x123"),
            Err(e) => error!("CAN TX failed: {:?}", e),
        }

        match twai.receive_async().await {
            Ok(frame) => match frame.id() {
                embedded_can::Id::Standard(id) => {
                    info!("CAN RX std id=0x{:03X} data={:?}", id.as_raw(), frame.data());
                }
                embedded_can::Id::Extended(id) => {
                    info!("CAN RX ext id=0x{:08X} data={:?}", id.as_raw(), frame.data());
                }
            },
            Err(_) => {}
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn task_rs485() -> ! {
    loop {
        info!("rs485 loop");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn task_ble() -> ! {
    loop {
        info!("ble loop");
        Timer::after(Duration::from_secs(1)).await;
    }
}