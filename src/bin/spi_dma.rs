#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new(p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA2_CH2, p.DMA2_CH1, spi_config);

    let mut cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);

    cortex_m::asm::delay(100_000);



    // enable conversion on ADXL362
    let write: [u8; 3] = [0x0A, 0x2D, 0x02];
    cs.set_low();
    spi.write(&write).await.ok();
    cs.set_high();
    cortex_m::asm::delay(1_000);
    loop {
        let mut read: [u8; 5] = [0x0B, 0x08, 0x00, 0x00, 0x00];
        cs.set_low();
        spi.transfer_in_place(&mut read).await.ok();
        cs.set_high();
        info!("xfer {=[u8]:x}", read);
    }
}
