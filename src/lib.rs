#![cfg_attr(not(test), no_std)]
pub mod config;
pub mod registers;

use core::fmt::Debug;

use crate::config::DRV8873Config;
use crate::registers::*;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{Operation, SpiDevice};

#[derive(core::fmt::Debug)]
pub enum Drv8873Error {
    Drv8873Fault(FaultStatus),
    SpiError(),
    SleepError(),
    InputError(&'static str),
}

#[derive(core::fmt::Debug)]
pub enum InputMode<PW: SetDutyCycle, P: StatefulOutputPin> {
    PhaseEnable(Option<(PW, P)>),
    PWM(Option<(PW, PW)>),
    IndependentHalfBridge(Option<(PW, PW)>),
    InputDisabled,
}

/// An instance of a DRV8873 device.
pub struct DRV8873<D: SpiDevice, F: Wait, P: StatefulOutputPin, PW: SetDutyCycle> {
    dev: D,
    fault_pin: Option<F>,
    sleep_pin: Option<P>,
    disable_pin: Option<P>,
    input_mode: InputMode<PW, P>,
    cfg: DRV8873Config,
}
impl<D: SpiDevice, F: Wait, P: StatefulOutputPin, PW: SetDutyCycle> DRV8873<D, F, P, PW> {
    /// Instanciates a new DRV8873 device with it's associated SPI interface.
    pub fn new(dev: D) -> Self {
        Self {
            dev,
            fault_pin: None,
            sleep_pin: None,
            disable_pin: None,
            input_mode: InputMode::PWM(None),
            cfg: DRV8873Config::default(),
        }
    }
    pub fn set_mode(&mut self, mode: InputMode<PW, P>) {}

    pub fn brake(&mut self, speed: u8) -> Result<(), Drv8873Error> {
        Ok(())
    }
    pub fn forward_with_speed(&mut self, speed: u8) -> Result<(), Drv8873Error> {
        match &mut self.input_mode {
            InputMode::PhaseEnable(Some((en, ph))) => {
                ph.set_high()
                    .map_err(|e| Drv8873Error::InputError("Unable to set PH_IN2"))?;
                en.set_duty_cycle_percent(speed)
                    .map_err(|_| Drv8873Error::InputError("Unable to set EN_IN1"))?;
            }
            InputMode::PhaseEnable(None) => {
                return Err(Drv8873Error::InputError(
                    "No pins assigned for motor control",
                ));
            }
            InputMode::PWM(Some((in1, in2))) => {
                in2.set_duty_cycle_fully_off()
                    .map_err(|e| Drv8873Error::InputError("Unable to set PH_IN2"))?;
                in1.set_duty_cycle_percent(speed)
                    .map_err(|_| Drv8873Error::InputError("Unable to set EN_IN1"))?;
            }
            InputMode::PWM(None) => {
                return Err(Drv8873Error::InputError(
                    "No pins assigned for motor control",
                ));
            }
            InputMode::IndependentHalfBridge(_) => todo!(),
            InputMode::InputDisabled => todo!(),
        }
        Ok(())
    }
    pub fn backward_with_speed(&mut self, speed: u8) -> Result<(), Drv8873Error> {
        match &mut self.input_mode {
            InputMode::PhaseEnable(Some((en, ph))) => {
                ph.set_low()
                    .map_err(|e| Drv8873Error::InputError("Unable to set PH_IN2"))?;
                en.set_duty_cycle_percent(speed)
                    .map_err(|_| Drv8873Error::InputError("Unable to set EN_IN1"))?;
            }
            InputMode::PhaseEnable(None) => {
                return Err(Drv8873Error::InputError(
                    "No pins assigned for motor control",
                ));
            }
            InputMode::PWM(Some((in1, in2))) => {
                in1.set_duty_cycle_fully_off()
                    .map_err(|_| Drv8873Error::InputError("Unable to set EN_IN1"))?;
                in2.set_duty_cycle_percent(speed)
                    .map_err(|e| Drv8873Error::InputError("Unable to set PH_IN2"))?;
            }
            InputMode::PWM(None) => {
                return Err(Drv8873Error::InputError(
                    "No pins assigned for motor control",
                ));
            }
            InputMode::IndependentHalfBridge(_) => todo!(),
            InputMode::InputDisabled => todo!(),
        }
        Ok(())
    }
    pub fn with_fault_pin(mut self, fault_pin: F) -> Self {
        self.fault_pin = Some(fault_pin);
        self
    }
    pub fn with_sleep_pin(mut self, sleep_pin: P) -> Self {
        self.sleep_pin = Some(sleep_pin);
        self
    }
    /// Assign a disable pin, when set high this pin disables the output drivers of the DRV8873.
    pub fn with_disable_pin(mut self, disable_pin: P) -> Self {
        self.disable_pin = Some(disable_pin);
        self
    }
    pub async fn set_config(&mut self) -> Result<(), Drv8873Error> {
        self.cfg.write_config(&mut self.dev).await?;
        Ok(())
    }
    /// Read the [FaultStatus] register from the device
    pub async fn get_fault(&mut self) -> Result<FaultStatus, Drv8873Error> {
        let (fs, _) = FaultStatus::read(&mut self.dev).await?;
        Ok(fs)
    }
    /// Read the [DiagnosticStatus] register from the device
    pub async fn get_diagnostics(&mut self) -> Result<DiagnosticStatus, Drv8873Error> {
        let (ds, _) = DiagnosticStatus::read(&mut self.dev).await?;
        Ok(ds)
    }
    #[inline]
    fn is_sleeping(&mut self) -> Result<bool, Drv8873Error> {
        if let Some(sleep) = &mut self.sleep_pin {
            sleep.is_set_low().map_err(|_| Drv8873Error::SleepError())
        } else {
            Ok(false)
        }
    }
    #[inline]
    /// Sets the nSLEEP pin high and waits for t_wake (1.5ms).
    async fn awaken(&mut self, delay: &mut impl DelayNs) -> Result<(), Drv8873Error> {
        if self.is_sleeping()? {
            if let Some(sleep) = &mut self.sleep_pin {
                sleep.set_high().map_err(|_| Drv8873Error::SleepError())?;
                delay.delay_us(1500).await;
            }
        }
        Ok(())
    }
    #[inline]
    /// Sets the nSLEEP pin low waits for t_sleep (50us).
    async fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), Drv8873Error> {
        if !self.is_sleeping()? {
            if let Some(sleep) = &mut self.sleep_pin {
                {
                    sleep.set_low().map_err(|_| Drv8873Error::SleepError())?;
                    delay.delay_us(50).await;
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;
    use registers::ReadableRegister;

    mod bar {
        use super::*;
        use embedded_hal_mock::common::Generic;
        use embedded_hal_mock::eh1::delay::NoopDelay;
        use embedded_hal_mock::eh1::digital::{
            Mock as WaitMock, State, Transaction as WaitTransaction,
        };
        use embedded_hal_mock::eh1::pwm::{Mock as PwmMock, Transaction as PwmTransaction};
        use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
        #[async_std::test]
        async fn registers() {
            //let cfg = DRV8873Config::default();
            let expectations = [
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![0b01000000, 0b00000000],
                    vec![0b11000000, 0b00000001],
                ),
                SpiTransaction::transaction_end(),
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![0b01000010, 0b00000000],
                    vec![0b11000000, 0b10000000],
                ),
                SpiTransaction::transaction_end(),
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![0b00000100, 0b11111111],
                    vec![0b11000000, 0b10000000],
                ),
                SpiTransaction::transaction_end(),
            ];
            let mut spi = SpiMock::new(&expectations);
            // let mut dev: DRV8873<Generic<SpiTransaction<u8>>, WaitMock, WaitMock> =
            //     DRV8873::new(spi.clone(), None, None);

            // dev.get_status().await;
            // dev.get_diagnostics().await;
            match FaultStatus::read(&mut spi).await {
                Ok((fs, None)) => assert!(fs.old()),
                Ok((_, _)) => {
                    panic!();
                }
                Err(_) => {
                    panic!();
                }
            }
            match DiagnosticStatus::read(&mut spi).await {
                Ok((ds, None)) => assert!(ds.ol1()),
                Ok((_, _)) => {
                    panic!();
                }
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
        #[async_std::test]
        async fn dev() {
            let expectations = [
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![0b01000000, 0b00000000],
                    vec![0b11000000, 0b00000001],
                ),
                SpiTransaction::transaction_end(),
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![0b01000010, 0b00000000],
                    vec![0b11000000, 0b10000000],
                ),
                SpiTransaction::transaction_end(),
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer(
                    vec![
                        0b00000100,
                        ControlRegister1::default().0,
                        0b00000110,
                        ControlRegister2::default().0,
                        0b00001000,
                        ControlRegister3::default().0,
                        0b00001010,
                        ControlRegister4::default().0,
                    ],
                    vec![
                        0b11000000, 0b10000000, 0b11000000, 0b10000000, 0b11000000, 0b10000000,
                        0b11000000, 0b10000000,
                    ],
                ),
                SpiTransaction::transaction_end(),
            ];

            let fault_expectation = [];
            let sleep_expectation = [
                WaitTransaction::get_state(State::Low),
                WaitTransaction::set(State::High),
                WaitTransaction::get_state(State::High),
                WaitTransaction::set(State::Low),
            ];
            let mut spi = SpiMock::new(&expectations);
            let mut fault = WaitMock::new(fault_expectation);
            let mut sleep = WaitMock::new(&sleep_expectation);
            let mut delay = NoopDelay::new();
            let mut dev: DRV8873<Generic<SpiTransaction<u8>>, WaitMock, WaitMock, PwmMock> =
                DRV8873::new(spi.clone())
                    .with_fault_pin(fault.clone())
                    .with_sleep_pin(sleep.clone());
            dev.awaken(&mut delay).await.unwrap();
            dev.get_fault().await.unwrap();
            dev.get_diagnostics().await.unwrap();
            dev.set_config().await.unwrap();
            dev.sleep(&mut delay).await.unwrap();
            drop(dev);
            spi.done();
            sleep.done();
            fault.done();
        }
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
