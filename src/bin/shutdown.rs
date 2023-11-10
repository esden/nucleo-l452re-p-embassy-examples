/*
 * This code uses serial output instead of RTT for debug output.
 * This is done because sleeping would disrupt RTT logging.
 *
 * To read the log output use the following command:
 * `socat /dev/ttyACM0,b115200,raw,echo=0 STDOUT | defmt-print -e target/thumbv7em-none-eabi/debug/shutdown`
 *
 * You will likely have to install socat and defmt-print for this.
 */

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Speed, Output, Level};
use embassy_stm32::{pac, bind_interrupts, usart, peripherals};
use embassy_stm32::pac::rtc::vals::Alrwf;
use embassy_stm32::rcc::{self, ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{Config as UartConfig, Uart};
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_serial as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    LPUART1 => usart::InterruptHandler<peripherals::LPUART1>;
});

fn log_state() {
    info!("RTC cr:  {:#06x} alrmr {:#06x}", pac::RTC.cr().read().0, pac::RTC.alrmr(0).read().0);
    info!("PWR cr1: {:#06x}", pac::PWR.cr1().read().0);
}

fn sleep() {
    // Configure backup domain
    info!("Setting PWR.cr1 setting shutdown low power mode");
    flush();
    pac::PWR.cr1().modify(|w| w.set_lpms(pac::pwr::vals::Lpms::SHUTDOWN));

    unsafe {
        let mut scb = cortex_m::Peripherals::steal().SCB;
        scb.set_sleepdeep();
        cortex_m::asm::dsb();
        cortex_m::asm::wfi();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = {
        let mut config = Config::default();
        config.rcc.mux = ClockSrc::PLL(
            PLLSource::HSI16,
            PLLClkDiv::Div4,
            PLLSrcDiv::Div1,
            PLLMul::Mul20,
            None,
        );
        config.rcc.lse = Some(Hertz(32_768));
        config.rcc.rtc_mux = rcc::RtcClockSource::LSE;
        embassy_stm32::init(config)
    };

    let mut led = Output::new(p.PB13, Level::High, Speed::Low);
    // Initialize debug UART
    let debug_uart_config = UartConfig::default();
    let debug_uart = Uart::new(p.LPUART1, p.PA3, p.PA2, Irqs, p.DMA2_CH6, NoDma, debug_uart_config).unwrap();
    defmt_serial::defmt_serial(debug_uart);
    info!("Hello World!");

    // Check if alarm flag was set
    if pac::RTC.isr().read().alrf(0) {
        info!("Alarm was triggered!");
        pac::RTC.isr().modify(|w| w.set_alrf(0, false));

        info!("Going back to sleep...");
        log_state();
        led.set_low();
        sleep();
    };

    info!("This is the first time waking up. So we need to set up a few things.");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());
    info!("Got RTC! {:?}", now.timestamp());

    rtc.set_datetime(now.into()).expect("datetime not set");

    // In reality the delay would be much longer
    info!("Waiting...");
    Timer::after(Duration::from_millis(1000)).await;

    let then: NaiveDateTime = rtc.now().unwrap().into();
    info!("Got RTC! {:?}", then.timestamp());

    // Allow write access to the RTC registers
    info!("Setting PWR.cr1");
    pac::PWR.cr1().modify(|w| w.set_dbp(true));
    pac::RTC.wpr().write(|w| w.0 = 0xCA);
    pac::RTC.wpr().write(|w| w.0 = 0x53);

    // Turn off alarm
    info!("Setting RTC.cr disabling alarm");
    pac::RTC.cr().modify(|w| w.set_alre(1, false));
    while pac::RTC.isr().read().alrwf(0) == Alrwf::UPDATENOTALLOWED {}
    // Let the alarm go off every minute at 05 seconds
    info!("Setting RTC.almr");
    pac::RTC.alrmr(0).modify(|w| {
        w.set_msk4(pac::rtc::vals::AlrmrMsk::NOTMASK); // Date/Day ignore
        //w.set_wdsel(pac::rtc::vals::AlrmrWdsel::DATEUNITS);
        //w.set_dt(0);
        //w.set_du(0);
        w.set_msk3(pac::rtc::vals::AlrmrMsk::NOTMASK); // Hours ignore
        //w.set_pm(pac::rtc::vals::AlrmrPm::AM);
        //w.set_ht(0);
        //w.set_hu(0);
        w.set_msk2(pac::rtc::vals::AlrmrMsk::NOTMASK); // Minutes ignore
        //w.set_mnt(0);
        //w.set_mnu(0);
        w.set_msk1(pac::rtc::vals::AlrmrMsk::MASK); // Seconds do not ignore
        w.set_st(0); // 0 second tens
        w.set_su(5); // 5 seconds units
    });
    // Enable alarm and alarm interrupt
    info!("Setting RTC.cr enabling alarm and interrupt");
    pac::RTC.cr().modify(|w| {
        w.set_alrie(0, true); // Enable alarm interrupt
        w.set_alre(0, true); // Enable alarm
    });

    info!("Now going to sleep for the first time...");
    log_state();
    led.set_low();
    sleep();

    info!("We ended up in the loop?");
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        info!("Loop...");
    }
}
