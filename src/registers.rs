use core::fmt::Debug;

use crate::Drv8873Error;
use bitfield::bitfield;
use embedded_hal_async::spi::Operation;
use embedded_hal_async::spi::SpiDevice;
use num_enum::{Default, FromPrimitive, IntoPrimitive};

bitfield! {
    pub struct CommandByte(u8);
    impl Debug;

    u8, address, set_address: 5, 1;
    read_bit, set_read_bit: 6;
}
impl CommandByte {
    /// Construct a write command byte to the specified address.
    pub fn write(addr: u8) -> Self {
        let mut cb = Self(0);
        cb.set_address(addr);
        cb
    }
    /// Construct a read command byte to the specified address.
    pub fn read(addr: u8) -> Self {
        let mut cb = Self(0);
        cb.set_read_bit(true);
        cb.set_address(addr);
        cb
    }
}
/// The status byte should have it's 2 most significant bits set, the other 6 correspond to
/// faults as described in the [FaultStatus] register.
pub(crate) fn get_status(status_byte: u8) -> Option<FaultStatus> {
    if status_byte != 0b11000000 {
        Some(FaultStatus(status_byte & 0b00111111))
    } else {
        None
    }
}
/// Implemented by all the registers as a wrapper to be able to implement the [ReadableRegister]
/// and [WriteableRegister] traits.
pub(crate) trait Register: Sized + Debug {
    const ADDR: u8;
    fn from_byte(byte: u8) -> Self;
    fn to_byte(&self) -> u8;
}
pub(crate) trait ReadableRegister<D>
where
    Self: Sized + Register,
    D: SpiDevice,
{
    /// Reads a register from the device and returns itself and the device status if any of the
    /// fault bits are set.
    #[inline]
    async fn read(dev: &mut D) -> Result<(Self, Option<FaultStatus>), Drv8873Error> {
        let mut buf: [u8; 2] = [0; 2];
        let cb = CommandByte::read(Self::ADDR);
        dev.transfer(&mut buf, &[cb.0, 0x00])
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        Ok((Self::from_byte(buf[1]), get_status(buf[0])))
    }
}
pub(crate) trait WriteableRegister<D>
where
    Self: Sized + Register,
    D: SpiDevice,
{
    #[inline]
    async fn write(&self, dev: &mut D) -> Result<(), Drv8873Error> {
        let mut buf: [u8; 2] = [0; 2];
        let cb = CommandByte::write(Self::ADDR);
        dev.transfer(&mut buf, &[cb.0, self.to_byte()])
            .await
            .map_err(|_| Drv8873Error::SpiError())?;
        Ok(())
    }
}
impl Register for FaultStatus {
    const ADDR: u8 = 0x00;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
#[allow(clippy::derivable_impls)]
impl Default for FaultStatus {
    fn default() -> Self {
        Self(0)
    }
}
impl Register for DiagnosticStatus {
    const ADDR: u8 = 0x01;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
#[allow(clippy::derivable_impls)]
impl Default for DiagnosticStatus {
    fn default() -> Self {
        Self(0)
    }
}
impl Register for ControlRegister1 {
    const ADDR: u8 = 0x02;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
impl Default for ControlRegister1 {
    fn default() -> Self {
        let mut cr1 = ControlRegister1(0);
        cr1.set_mode(Mode::PWM);
        cr1.set_sr(RiseTime::VoltPerUs10_8);
        cr1.set_toff(Toff::Us40);
        cr1
    }
}
impl Register for ControlRegister2 {
    const ADDR: u8 = 0x03;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
impl Default for ControlRegister2 {
    fn default() -> Self {
        let mut cr2 = ControlRegister2(0);
        cr2.set_ocp_mode(OcpMode::LatchedFault);
        cr2.set_ocp_t_retry(OcpTRetry::Ms4);
        cr2
    }
}
impl Register for ControlRegister3 {
    const ADDR: u8 = 0x04;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
impl Default for ControlRegister3 {
    fn default() -> Self {
        let mut cr3 = ControlRegister3(0);
        cr3.set_lock(Lock::Unlocked);
        cr3
    }
}
impl Register for ControlRegister4 {
    const ADDR: u8 = 0x05;
    #[inline]
    fn from_byte(byte: u8) -> Self {
        Self(byte)
    }
    #[inline]
    fn to_byte(&self) -> u8 {
        self.0
    }
}
impl Default for ControlRegister4 {
    fn default() -> Self {
        let mut cr4 = ControlRegister4(0);
        cr4.set_i_trip_lvl(ITripLvl::Ampere6_5);
        cr4
    }
}
impl<D: SpiDevice> ReadableRegister<D> for FaultStatus {}
impl<D: SpiDevice> ReadableRegister<D> for DiagnosticStatus {}
impl<D: SpiDevice> ReadableRegister<D> for ControlRegister1 {}
impl<D: SpiDevice> ReadableRegister<D> for ControlRegister2 {}
impl<D: SpiDevice> ReadableRegister<D> for ControlRegister3 {}
impl<D: SpiDevice> ReadableRegister<D> for ControlRegister4 {}
impl<D: SpiDevice> WriteableRegister<D> for ControlRegister1 {}
impl<D: SpiDevice> WriteableRegister<D> for ControlRegister2 {}
impl<D: SpiDevice> WriteableRegister<D> for ControlRegister3 {}
impl<D: SpiDevice> WriteableRegister<D> for ControlRegister4 {}
bitfield! {
    pub struct FaultStatus(u8);
    impl Debug;

    /// Open-load detection flag
    pub old, _ : 0;
    /// Overtemperature shutdown flag
    pub tsd, _ : 1;
    /// Overcurrent condition flag
    pub ocp, _ : 2;
    /// Charge-pump undervoltage fault condition flag
    pub cpuv, _ : 3;
    /// UVLO fault condition flag
    pub uvlo, _ : 4;
    /// Overtemperature warning flag
    pub otw, _ : 5;
    /// Global FAULT status register. Compliments the nFAULT pin
    pub fault, _ : 6;
    /// Reserved
    _, _ : 7;
}

bitfield! {
    pub struct DiagnosticStatus(u8);
    impl Debug;

    /// Indicates overcurrent fault on the low-side FET of half bridge 2
    pub ocp_l2, _ : 0;
    ///Indicates overcurrent fault on the high-side FET of half bridge 2
    pub ocp_h2, _ : 1;
    /// Indicates overcurrent fault on the low-side FET of half bridge 1
    pub ocp_l1, _ : 2;
    /// Indicates overcurrent fault on the high-side FET of half bridge 1
    pub ocp_h1, _ : 3;
    /// Indicates output 2 is in current regulation
    pub itrip2, _ : 4;
    /// Indicates output 1 is in current regulation
    pub itrip1, _ : 5;
    /// Indicates open-load detection on half bridge 2
    pub ol2, _ : 6;
    /// Indicates open-load detection on half bridge 1
    pub ol1, _ : 7;
}

bitfield! {
    pub struct ControlRegister1(u8);
    impl Debug;

    /// Set the input [Mode]
    pub from into Mode,     mode, set_mode : 1, 0;
    /// Set the [RiseTime]
    pub from into RiseTime, sr, set_sr : 4, 2;
    /// Sets whether the outputs follow the input pins (0) or the SPI registers [ControlRegister3.en_in1()] and
    /// [ControlRegister3.ph_in2()]
    pub spi_in, set_spi_in : 5;
    /// Sets the off time [Toff]
    pub from into Toff, toff, set_toff: 7,6;
}
bitfield! {
    pub struct ControlRegister2(u8);
    impl Debug;

    /// Overcurrent condition handling mode.
    pub from into OcpMode, ocp_mode, set_ocp_mode : 1, 0;
    /// Overcurrent retry time.
    pub from into OcpTRetry, ocp_t_retry, set_ocp_t_retry : 3, 2;
    /// Disable charge-pump undervoltage fault.
    pub dis_cpuv, set_dis_cpuv: 4;
    /// OTW (Overtemperature Warning) condition handling, when set OTW is reported on nFAULT and the FAULT bit.
    pub otw_rep, set_otw_rep: 5;
    /// Overtemperature condition handling mode (0 = latched fault, 1 = automatic recovery).
    pub tsd_mode, set_tsd_mode: 6;
    /// ITRIP condition handling, when set ITRIP is reported on nFAULT and the FAULT bit.
    pub itrip_rep, set_itrip_rep: 7;
}
bitfield! {
    pub struct ControlRegister3(u8);
    impl Debug;

    /// EN/IN1 bit to control the outputs through SPI (when SPI_IN = 1b)
    pub ph_in2, set_ph_in2: 0;
    /// PH/IN2 bit to control the outputs through SPI (when SPI_IN = 1b)
    pub en_in1, set_en_in1: 1;
    /// Enabled only in the Independent PWM mode
    /// 0b = Half bridge 2 enabled
    /// 1b = Half bridge 2 disabled (Hi-Z)
    pub out2_dis, set_out2_dis: 2;
    /// Enabled only in the Independent PWM mode
    /// 0b = Half bridge 1 enabled
    /// 1b = Half bridge 1 disabled (Hi-Z)
    pub out1_dis, set_out1_dis: 3;
    /// Write 011b [Lock::Locked] to this register to lock all register settings in the IC1
    /// control register except to these bits and address 0x04, bit 7 (CLR_FLT)
    /// Write 100b [Lock::Unlocked] to this register to unlock all register settings in the
    /// IC1 control register
    pub from into Lock, lock, set_lock: 6, 4;
    /// Write a 1b to this bit to clear the fault bits. This bit is
    /// automatically reset after a write.
    pub clr_flt, set_clr_flt: 7;
}
bitfield! {
    pub struct ControlRegister4(u8);
    impl Debug;

    /// Disable/enable current regulation with [ITrip]
    pub from into ITrip, i_trip, set_i_trip: 1,0;
    /// Set the current limit to [ITripLvl]
    pub from into ITripLvl, i_trip_lvl, set_i_trip_lvl: 3,2;
    /// Enable open load diagnostics in active mode
    pub en_ola, set_en_ola: 4;
    /// Set open load diagnostics delay (0 = 300us, 1 = 1.2ms);
    pub olp_dly, set_old_dly: 5;
    /// Write 1b to run open load diagnostic in standby mode. When
    /// open load test is complete EN_OLP returns to 0b (status check)
    pub en_olp, set_en_olp: 6;
    /// Reserved
    rsvd, _ : 7;
}
/// Determines the Toff time set in [ControlRegister1]
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum Toff {
    Us20 = 0b00,
    #[default]
    Us40 = 0b01,
    Us60 = 0b10,
    Us80 = 0b11,
}

/// Determines the rise time set in [ControlRegister1]
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum RiseTime {
    VoltPerUs53_2 = 0b000,
    VoltPerUs34_0 = 0b001,
    VoltPerUs18_3 = 0b010,
    VoltPerUs13_0 = 0b011,
    #[default]
    VoltPerUs10_8 = 0b100,
    VoltPerUs7_9 = 0b101,
    VoltPerUs5_3 = 0b110,
    VoltPerUs2_6 = 0b111,
}

/// Determines the device mode set in [ControlRegister1]
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum Mode {
    PhaseEnable = 0b00,
    #[default]
    PWM = 0b01,
    IndependentHalfBridge = 0b10,
    InputDisabled = 0b11,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum OcpMode {
    #[default]
    LatchedFault = 0b00,
    AutomaticRetry = 0b01,
    ReportOnly = 0b10,
    NoAction = 0b11,
}
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum OcpTRetry {
    Ms0_5 = 0b00,
    Ms1 = 0b01,
    Ms2 = 0b10,
    #[default]
    Ms4 = 0b11,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum Lock {
    #[default]
    Unlocked = 0b100,
    Locked = 0b011,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum ITrip {
    #[default]
    Enabled = 0b00,
    Out1Disabled = 0b01,
    Out2Disabled = 0b10,
    Disabled = 0b11,
}
#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive, Default)]
pub enum ITripLvl {
    Ampere4 = 0b00,
    Ampere5_4 = 0b01,
    #[default]
    Ampere6_5 = 0b10,
    Ampere7 = 0b11,
}
