use defmt::Format;
use defmt::Formatter;
use embedded_hal_async::spi::Operation;
use embedded_hal_async::spi::SpiDevice;

use crate::registers::get_status;
use crate::{
    registers::{CommandByte, ReadableRegister, Register},
    registers::{ControlRegister1, ControlRegister2, ControlRegister3, ControlRegister4},
    Drv8873Error,
};
/// Holds the 4 control registers of the DRV8873 IC and associated functions to read and write
/// them. Can be used to initialize a [DRV8873] to something other than default settings.
#[derive(Debug, Default)]
pub struct DRV8873Config {
    pub cr1: ControlRegister1,
    pub cr2: ControlRegister2,
    pub cr3: ControlRegister3,
    pub cr4: ControlRegister4,
}
impl Clone for DRV8873Config {
    fn clone(&self) -> Self {
        Self {
            cr1: ControlRegister1::from_byte(self.cr1.0),
            cr2: ControlRegister2::from_byte(self.cr2.0),
            cr3: ControlRegister3::from_byte(self.cr3.0),
            cr4: ControlRegister4::from_byte(self.cr4.0),
        }
    }
}
impl Format for DRV8873Config {
    fn format(&self, fmt: Formatter) {
        defmt::write!(
            fmt,
            "{},{},{},{}",
            self.cr1.0,
            self.cr2.0,
            self.cr3.0,
            self.cr4.0
        )
    }
}

impl PartialEq for DRV8873Config {
    fn eq(&self, other: &Self) -> bool {
        self.cr1.0 == other.cr1.0
            && self.cr2.0 == other.cr2.0
            && self.cr3.0 == other.cr3.0
            && self.cr4.0 == other.cr4.0
    }
}

impl DRV8873Config {
    pub(crate) async fn write_config(&self, dev: &mut impl SpiDevice) -> Result<(), Drv8873Error> {
        let mut buf = [CommandByte::write(ControlRegister1::ADDR).0, self.cr1.0];
        dev.transfer_in_place(&mut buf)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        let mut buf = [CommandByte::write(ControlRegister2::ADDR).0, self.cr2.0];
        dev.transfer_in_place(&mut buf)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        let mut buf = [CommandByte::write(ControlRegister3::ADDR).0, self.cr3.0];
        dev.transfer_in_place(&mut buf)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        let mut buf = [CommandByte::write(ControlRegister4::ADDR).0, self.cr4.0];
        dev.transfer_in_place(&mut buf)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        // buf.chunks(2).try_for_each(|c| {
        //     if let Some(status) = get_status(c[0]) {
        //         Err(Drv8873Error::Drv8873Fault(status))
        //     } else {
        //         Ok(())
        //     }
        // })
        Ok(())
    }
    pub(crate) async fn read_config(dev: &mut impl SpiDevice) -> Result<Self, Drv8873Error> {
        let cfg = Self {
            cr1: ControlRegister1::read(dev).await?,
            cr2: ControlRegister2::read(dev).await?,
            cr3: ControlRegister3::read(dev).await?,
            cr4: ControlRegister4::read(dev).await?,
        };
        Ok(cfg)
    }
}
