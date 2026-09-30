// src/app/task_rtc.rs
use esp_hal::i2c::master::I2c;
use esp_hal::gpio::Input;
use defmt::{error, info, Debug2Format};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_futures::select::{select, Either};
use time::PrimitiveDateTime;

pub static RTC_TIME_CHANNEL: Channel<CriticalSectionRawMutex, u64, 1> = Channel::new();
pub static SET_TIME_CHANNEL: Channel<CriticalSectionRawMutex, u64, 1> = Channel::new();

#[embassy_executor::task]
pub async fn task_rtc(
    i2c: I2c<'static, esp_hal::Async>,
    mut rtc_int_pin: Input<'static>,
) -> ! {
    let mut rtc = pcf85063a::PCF85063::new(i2c);

    // // 上电初始时间
    // let now = PrimitiveDateTime::new(
    //     Date::from_calendar_date(2021, Month::April, 4).unwrap(),
    //     Time::from_hms(16, 52, 0).unwrap(),
    // );
    // rtc.set_datetime(&now).await.unwrap();
    
    // 开1HZ脉冲INT， TCF=1Hz, TE=1, TIE=1, TI_TP=0 → 0x16
    rtc.write_register(0x11, 0x16).await.unwrap();
    rtc.write_register(0x10, 0x01).await.unwrap();

    loop {
        match select(
            SET_TIME_CHANNEL.receive(),
            rtc_int_pin.wait_for_low(), // 等待 INT 引脚变低（每秒一次）
        ).await {
            // 收到设置时间的请求
            Either::First(ts) => {
                match time::OffsetDateTime::from_unix_timestamp(ts as i64) {
                    Ok(odt) => {
                        let new_dt = PrimitiveDateTime::new(odt.date(), odt.time());
                        match rtc.set_datetime(&new_dt).await {
                            Ok(_) => {
                                info!("RTC set to unix={}", ts);
                                if let Ok(t) = rtc.get_datetime().await {
                                    let new_ts = t.assume_utc().unix_timestamp() as u64;
                                    let _ = RTC_TIME_CHANNEL.try_send(new_ts);
                                }
                            }
                            Err(e) => error!("RTC set_datetime failed: {:?}", Debug2Format(&e)),
                        }
                    }
                    Err(e) => error!("invalid unix ts: {:?}", Debug2Format(&e)),
                }
            }
            // INT 引脚脉冲到来，说明又过了一秒
            Either::Second(_) => {
                // 读秒寄存器，顺带清除 Timer Flag
                let ctrl2 = rtc.read_register(0x01).await.unwrap();
                rtc.write_register(0x01, ctrl2 & !(1 << 3)).await.unwrap();

                let t = rtc.get_datetime().await.unwrap();
                let ts = t.assume_utc().unix_timestamp() as u64;
                let _ = RTC_TIME_CHANNEL.try_send(ts);

                // info!("one secound");
            }
        }
    }
}