#![cfg(unix)]
#![cfg(test)]

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
        // let mut dev: DRV8873<Generic<SpiTransaction<u8>>, WaitMock, WaitMock> =
        //     DRV8873::new(spi.clone(), None, None);

        // dev.get_status().await;
        // dev.get_diagnostics().await;
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
    #[cfg(not(feature = "sleep"))]
    #[async_std::test]
    async fn dev() {
        let expectations = [
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b01000000, 0b00000000], vec![0b11000000, 0b00000001]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b01000010, 0b00000000], vec![0b11000000, 0b10000000]),
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
        // let mut sleep = WaitMock::new(&sleep_expectation);
        let mut delay = NoopDelay::new();
        let cfg = DRV8873Config::default();
        let mut dev: DRV8873<Generic<SpiTransaction<u8>>, WaitMock, WaitMock, NoopDelay> =
            DRV8873::new(spi.clone());
        dev.read_fault().await.unwrap();
        dev.read_diagnostics().await.unwrap();
        dev.write_config(&cfg).await.unwrap();
        drop(dev);
        spi.done();
        // sleep.done();
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
