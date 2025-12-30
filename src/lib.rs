//! # DRV8873
//!
//! An `async` library for interacting with the Texas Instruments [DRV8873SPW](https://www.ti.com/product/DRV8873) 40-V, 10-A H-Bridge
//! motor driver.
//!
#![cfg_attr(not(test), no_std)]
pub mod config;
pub mod registers;
mod tests;

#[doc(inline)]
pub use crate::{
    config::DRV8873Config,
    registers::{
        ControlRegister1, ControlRegister2, ControlRegister3, ControlRegister4, DisITrip, ITripLvl,
        Lock, Mode, OcpMode, OcpTRetry, RiseTime, Toff,
    },
};

use crate::registers::*;
use embedded_hal::digital::StatefulOutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

#[derive(core::fmt::Debug)]
pub enum Drv8873Error {
    Drv8873Fault(FaultStatus),
    SpiError(),
    SleepError(),
    InputError(&'static str),
}

/// An instance of a DRV8873 device.
pub struct DRV8873<D: SpiDevice, P: StatefulOutputPin> {
    dev: D,
    fault_pin: Option<P>,
    disable_pin: Option<P>,
    n_sleep_pin: Option<P>,
}

impl<D: SpiDevice, P: StatefulOutputPin> DRV8873<D, P> {
    pub fn new(dev: D) -> Self {
        Self {
            dev,
            fault_pin: None,
            disable_pin: None,
            n_sleep_pin: None,
        }
    }
    /// Assign a disable pin, when set high this pin disables the output drivers of the DRV8873.
    pub fn with_disable_pin(mut self, disable_pin: P) -> Self {
        self.disable_pin = Some(disable_pin);
        self
    }
    /// Read all the control registers from the device as a [DRV8873Config].
    pub async fn read_config(&mut self) -> Result<DRV8873Config, Drv8873Error> {
        DRV8873Config::read_config(&mut self.dev).await
    }
    /// Write a [DRV8873Config] to the device.
    pub async fn write_config(&mut self, cfg: &DRV8873Config) -> Result<(), Drv8873Error> {
        cfg.write_config(&mut self.dev).await?;
        Ok(())
    }
    /// Reads all the control registers from the device as a [DRV8873Config] and allows them to be
    /// modified through an [FnMut]
    pub async fn modify_config(
        &mut self,
        f: fn(DRV8873Config) -> DRV8873Config,
    ) -> Result<DRV8873Config, Drv8873Error>
where {
        let cfg = f(self.read_config().await?);
        cfg.write_config(&mut self.dev).await?;

        Ok(cfg)
    }
    /// Read the [FaultStatus] register from the device
    pub async fn read_fault(&mut self) -> Result<FaultStatus, Drv8873Error> {
        FaultStatus::read(&mut self.dev).await
    }
    /// Read the [DiagnosticStatus] register from the device
    pub async fn read_diagnostics(&mut self) -> Result<DiagnosticStatus, Drv8873Error> {
        DiagnosticStatus::read(&mut self.dev).await
    }
    /// Reads [ControlRegister1] from the device, returns an error if the status byte in the SPI
    /// response contains a fault.
    pub async fn read_cr1(&mut self) -> Result<ControlRegister1, Drv8873Error> {
        ControlRegister1::read(&mut self.dev).await
    }
    /// Reads [ControlRegister1] from the device and allows modification through a closure.
    pub async fn modify_cr1(
        &mut self,
        f: fn(ControlRegister1) -> ControlRegister1,
    ) -> Result<ControlRegister1, Drv8873Error> {
        let cr = f(self.read_cr1().await?);
        cr.write(&mut self.dev).await?;

        Ok(cr)
    }
    /// Reads [ControlRegister2] from the device, returns an error if the status byte in the SPI
    /// response contains a fault.
    pub async fn read_cr2(&mut self) -> Result<ControlRegister2, Drv8873Error> {
        ControlRegister2::read(&mut self.dev).await
    }
    /// Reads [ControlRegister2] from the device and allows modification through a closure.
    pub async fn modify_cr2(
        &mut self,
        f: fn(ControlRegister2) -> ControlRegister2,
    ) -> Result<ControlRegister2, Drv8873Error> {
        let cr = f(self.read_cr2().await?);
        cr.write(&mut self.dev).await?;

        Ok(cr)
    }
    /// Reads [ControlRegister3] from the device, returns an error if the status byte in the SPI
    /// response contains a fault.
    pub async fn read_cr3(&mut self) -> Result<ControlRegister3, Drv8873Error> {
        ControlRegister3::read(&mut self.dev).await
    }
    /// Reads [ControlRegister3] from the device and allows modification through a closure.
    pub async fn modify_cr3(
        &mut self,
        f: fn(ControlRegister3) -> ControlRegister3,
    ) -> Result<ControlRegister3, Drv8873Error> {
        let cr = f(self.read_cr3().await?);
        cr.write(&mut self.dev).await?;

        Ok(cr)
    }
    /// Reads [ControlRegister4] from the device, returns an error if the status byte in the SPI
    /// response contains a fault.
    pub async fn read_cr4(&mut self) -> Result<ControlRegister4, Drv8873Error> {
        ControlRegister4::read(&mut self.dev).await
    }
    /// Reads [ControlRegister4] from the device and allows modification through a closure.
    pub async fn modify_cr4(
        &mut self,
        f: fn(ControlRegister4) -> ControlRegister4,
    ) -> Result<ControlRegister4, Drv8873Error> {
        let cr = f(self.read_cr4().await?);
        cr.write(&mut self.dev).await?;

        Ok(cr)
    }

    pub async fn clear_fault(&mut self) -> Result<(), Drv8873Error> {
        self.modify_cr3(|mut cr3| {
            cr3.set_clr_flt(true);
            cr3
        })
        .await?;
        Ok(())
    }
}
