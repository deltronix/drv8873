#![cfg_attr(not(test), no_std)]
pub mod config;
pub mod registers;

use crate::config::DRV8873Config;
use crate::registers::*;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

pub struct DRV8873<D: SpiDevice, F: Wait, P: StatefulOutputPin> {
    dev: D,
    fault_pin: Option<F>,
    sleep_pin: Option<P>,
    fs: FaultStatus,
    ds: DiagnosticStatus,
    config: DRV8873Config,
}
impl<D: SpiDevice, F: Wait, P: StatefulOutputPin> DRV8873<D, F, P> {
    pub fn new(dev: D, fault_pin: Option<F>, sleep_pin: Option<P>) -> Self {
        Self {
            dev,
            fault_pin,
            sleep_pin,
            fs: FaultStatus(0),
            ds: DiagnosticStatus(0),
            config: DRV8873Config::default(),
        }
    }
    pub async fn init(&mut self) -> Result<(), Drv8873Error<D::Error>> {
        self.config.cr1.write(&mut self.dev).await?;
        self.config.cr2.write(&mut self.dev).await?;
        self.config.cr3.write(&mut self.dev).await?;
        self.config.cr4.write(&mut self.dev).await?;
        Ok(())
    }

    /// Sets the nSLEEP pin high and waits for t_wake (1.5ms).
    async fn awaken(&mut self, delay: &mut impl DelayNs) -> Result<(), Drv8873Error<D::Error>> {
        if let Some(sleep) = &mut self.sleep_pin {
            sleep.set_high().map_err(|_| Drv8873Error::SleepError())?;
            delay.delay_us(1500).await;
        }
        Ok(())
    }
    /// Sets the nSLEEP pin low waits for t_sleep (50us).
    async fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), Drv8873Error<D::Error>> {
        if let Some(sleep) = &mut self.sleep_pin {
            sleep.set_low().map_err(|_| Drv8873Error::SleepError())?;
            delay.delay_us(50).await;
        }
        Ok(())
    }
}
#[derive(core::fmt::Debug)]
pub enum Drv8873Error<D: embedded_hal_async::spi::Error> {
    Drv8873Fault(FaultStatus),
    SpiError(D),
    SleepError(),
}

#[cfg(test)]
mod tests {

    use super::*;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use registers::ReadableRegister;

    #[async_std::test]
    async fn device() {
        let expectations = [
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b01000000, 0b00000000], vec![0b11000000, 0b00000001]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b01000010, 0b00000000], vec![0b11000000, 0b10000000]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b00000100, 0b11111111], vec![0b11000000, 0b10000000]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi = SpiMock::new(&expectations);
        match FaultStatus::read(&mut spi).await {
            Ok(fs) => assert!(fs.old()),
            Err(_) => {
                panic!();
            }
        }
        match DiagnosticStatus::read(&mut spi).await {
            Ok(ds) => assert!(ds.ol1()),
            Err(_) => {
                panic!();
            }
        }
        let cr = ControlRegister1(0xFF);
        match cr.write(&mut spi).await {
            Ok(_) => {}
            Err(_) => {
                panic!();
            }
        }
        spi.done();
    }
    #[test]
    fn fault_status_register() {
        let fsr = FaultStatus(0b01010101);
        assert!(fsr.old());
        assert!(!fsr.tsd());
        assert!(fsr.ocp());
    }
    #[test]
    fn control_register1() {
        let mut cr1 = ControlRegister1(0u8);
        assert_eq!(cr1.mode(), Mode::PhaseEnable);
        assert_eq!(cr1.sr(), RiseTime::VoltPerUs53_2);

        cr1.set_sr(RiseTime::VoltPerUs7_9);
        cr1.set_mode(Mode::PWM);

        assert_eq!(cr1.sr(), RiseTime::VoltPerUs7_9);
        assert_eq!(cr1.mode(), Mode::PWM);
    }
    #[test]
    fn control_register2() {}
}
