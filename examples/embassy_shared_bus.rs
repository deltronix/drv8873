#![no_std]
#![no_main]
#![cfg(not(feature = "sleep"))]

use defmt::*;
use drv8873::registers::{ControlRegister1, ITripLvl, RiseTime};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{EXTI0, TIM1};
use embassy_stm32::spi::{Config, Mode, Spi, MODE_3};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::low_level::{
    CountingMode, OutputCompareMode, OutputPolarity, Timer as LlTimer,
};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel};
use embassy_stm32::timer::Channel::{self, Ch1};
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};

use drv8873::config::DRV8873Config;
use drv8873::DRV8873;

use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Set up the SPI bus
    let mut config = Config::default();
    config.frequency = Hertz(1_000_000);
    config.mode = MODE_3; // PROBABLY INCORRECT!
    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<'static, Async>>> = StaticCell::new();
    let spi_bus = SPI_BUS.init_with(|| {
        Mutex::new(Spi::new(
            p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH4, config,
        ))
    });
    let cs = Output::new(p.PA4, Level::High, Speed::High);
    let mut dev = SpiDevice::new(spi_bus, cs);

    let in1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let in2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    // let tim = LlTimer::new(p.TIM1);
    // tim.set_frequency(Hertz(500));
    // tim.set_counting_mode(CountingMode::EdgeAlignedUp);
    // tim.set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
    // tim.set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
    // tim.set_output_polarity(Channel::Ch1, OutputPolarity::ActiveLow);
    // tim.set_output_polarity(Channel::Ch2, OutputPolarity::ActiveLow);

    // let pwm = ComplementaryPwm::new(
    //     p.TIM1,
    //     Some(in1),
    //     None,
    //     Some(in2),
    //     None,
    //     None,
    //     None,
    //     None,
    //     None,
    //     Hertz(50000),
    //     embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
    // );
    //

    let pwm = SimplePwm::new(
        p.TIM1,
        Some(in1),
        Some(in2),
        None,
        None,
        Hertz(1000),
        CountingMode::EdgeAlignedUp,
    )
    .split();

    let pwm1 = pwm.ch1;
    let pwm2 = pwm.ch2;

    let sw1 = Input::new(p.PB3, Pull::None);
    let sw2 = Input::new(p.PB4, Pull::None);

    let fault = ExtiInput::new(p.PB0, p.EXTI0, embassy_stm32::gpio::Pull::None);
    let sleep = Output::new(p.PA0, Level::High, Speed::High);

    let mut config = DRV8873Config::default();
    config.cr1.set_sr(RiseTime::VoltPerUs13_0);
    config.cr2.set_itrip_rep(true);
    config.cr4.set_i_trip_lvl(ITripLvl::Ampere4);

    let mut drv: DRV8873<&mut SpiDevice<'_, NoopRawMutex, Spi<'_, Async>, Output<'_>>, Output<'_>> =
        DRV8873::new(&mut dev);
    drv.write_config(&config);

    unwrap!(spawner.spawn(fault_handler(fault)));
    loop {
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn fault_handler(mut fault: ExtiInput<'static>) {
    fault.wait_for_falling_edge().await;
}
