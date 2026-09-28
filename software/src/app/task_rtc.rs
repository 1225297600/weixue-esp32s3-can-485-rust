use esp_hal::i2c::master::I2c;
use embassy_time::{Duration, Timer};
use defmt::{error, info, Debug2Format};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_futures::select::{select, Either};

pub static RTC_TIME_CHANNEL: Channel<CriticalSectionRawMutex, u64, 1> = Channel::new();
pub static SET_TIME_CHANNEL: Channel<CriticalSectionRawMutex, u64, 1> = Channel::new();

#[embassy_executor::task]
pub async fn task_rtc(i2c: I2c<'static, esp_hal::Async>) -> ! {
    let mut rtc = pcf85063a::PCF85063::new(i2c);

    use time::{Date, Month, PrimitiveDateTime, Time};

    let now = PrimitiveDateTime::new(
        Date::from_calendar_date(2021, Month::April, 4).unwrap(),
        Time::from_hms(16, 52, 0).unwrap(),
    );
    rtc.set_datetime(&now).await.unwrap();

    loop {
        match select(
            SET_TIME_CHANNEL.receive(),        // Future 1: 有设置请求就返回 u64
            Timer::after(Duration::from_secs(1)), // Future 2: 1秒后返回 ()
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
            // 1秒定时到了，正常读取 RTC 推给网页
            Either::Second(_) => {
                let t = rtc.get_datetime().await.unwrap();
                let ts = t.assume_utc().unix_timestamp() as u64;
                let _ = RTC_TIME_CHANNEL.try_send(ts);
            }
        }
    }
}