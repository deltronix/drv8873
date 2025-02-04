#![cfg_attr(not(test), no_std)]
pub mod registers;

use core::marker::PhantomData;

use crate::registers::*;
use bitfield::bitfield;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use num_enum::{FromPrimitive, IntoPrimitive};

struct DRV8873<D: SpiDevice, F: Wait, P: StatefulOutputPin> {
    dev: D,
    fault_pin: Option<F>,
    sleep_pin: Option<P>,
    fs: FaultStatus,
    ds: DiagnosticStatus,
    config: DRV8873Config,
}
#[derive(Default)]
struct DRV8873Config {
    cr1: ControlRegister1,
    cr2: ControlRegister2,
    cr3: ControlRegister3,
    cr4: ControlRegister4,
}

impl<D: SpiDevice, F: Wait, P: StatefulOutputPin> DRV8873<D, F, P> {
    fn new(dev: D, fault_pin: Option<F>, sleep_pin: Option<P>) -> Self {
        Self {
            dev,
            fault_pin,
            sleep_pin,
            fs: FaultStatus(0),
            ds: DiagnosticStatus(0),
            config: DRV8873Config::default(),
        }
    }
    async fn init(&mut self) -> Result<(), Error<D::Error>> {
        self.config.cr1.write(&mut self.dev).await?;
        self.config.cr2.write(&mut self.dev).await?;
        self.config.cr3.write(&mut self.dev).await?;
        self.config.cr4.write(&mut self.dev).await?;
        Ok(())
    }
}

enum Error<D: embedded_hal_async::spi::Error> {
    Drv8873Fault(FaultStatus),
    SpiError(D),
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
            SpiTransaction::transfer(vec![0b00000000, 0b00000000], vec![0b11000000, 0b00000001]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(vec![0b00000010, 0b00000000], vec![0b11000000, 0b10000000]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi = SpiMock::new(&expectations);
        match FaultStatus::read(&mut spi).await {
            Ok(fs) => assert!(fs.old()),
            Err(e) => match e {
                Error::Drv8873Fault(fs) => {
                    println!("Fault: {:?}", fs);
                    panic!();
                }
                Error::SpiError(_) => {
                    println!("SpiError");
                    panic!();
                }
            },
        }
        match DiagnosticStatus::read(&mut spi).await {
            Ok(ds) => assert!(ds.ol1()),
            Err(e) => match e {
                Error::Drv8873Fault(fs) => {
                    println!("Fault: {:?}", fs);
                    panic!();
                }
                Error::SpiError(_) => {
                    println!("SpiError");
                    panic!();
                }
            },
        }
        // let cr = ControlRegister1(0xFF);
        // match cr.write(&mut spi).await {
        //     Ok(_) => {}
        //     Err(_) => {}
        // }
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
