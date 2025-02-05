#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::spi::{Config, Spi, MODE_3};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;

use drv8873::config::DRV8873Config;
use drv8873::DRV8873;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut config = Config::default();
    config.frequency = Hertz(4_000_000);
    config.mode = MODE_3; // PROBABLY INCORRECT!
    let mut spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH4, config);

    let drv8873_config = DRV8873Config::default();

    let drv8873 = DRV8873::new(&mut spi, None, None);
    drv8873.init();
}
