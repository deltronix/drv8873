use embedded_hal_async::spi::Operation;
use embedded_hal_async::spi::SpiDevice;

use crate::registers::get_status;
use crate::{
    registers::{CommandByte, Register},
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

impl DRV8873Config {
    pub(crate) async fn write_config(&self, dev: &mut impl SpiDevice) -> Result<(), Drv8873Error> {
        let mut rx_buf: [u8; 8] = [0; 8];
        let tx_buf = [
            CommandByte::write(ControlRegister1::ADDR).0,
            self.cr1.0,
            CommandByte::write(ControlRegister2::ADDR).0,
            self.cr2.0,
            CommandByte::write(ControlRegister3::ADDR).0,
            self.cr3.0,
            CommandByte::write(ControlRegister4::ADDR).0,
            self.cr4.0,
        ];
        let mut ops = [Operation::Transfer(&mut rx_buf, &tx_buf)];
        dev.transaction(&mut ops)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;

        rx_buf.chunks(2).try_for_each(|c| {
            if let Some(status) = get_status(c[0]) {
                Err(Drv8873Error::Drv8873Fault(status))
            } else {
                Ok(())
            }
        })
    }
    pub(crate) async fn read_config(&self, dev: &mut impl SpiDevice) -> Result<Self, Drv8873Error> {
        let mut buf = [
            CommandByte::read(ControlRegister1::ADDR).0,
            0x00,
            CommandByte::read(ControlRegister2::ADDR).0,
            0x00,
            CommandByte::read(ControlRegister3::ADDR).0,
            0x00,
            CommandByte::read(ControlRegister4::ADDR).0,
            0x00,
        ];
        let mut ops = [Operation::TransferInPlace(&mut buf)];
        dev.transaction(&mut ops)
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        let cfg = Self {
            cr1: ControlRegister1::from_byte(buf[1]),
            cr2: ControlRegister2::from_byte(buf[3]),
            cr3: ControlRegister3::from_byte(buf[5]),
            cr4: ControlRegister4::from_byte(buf[7]),
        };
        Ok(cfg)
    }
}
