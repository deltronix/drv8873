use crate::registers::{ControlRegister1, ControlRegister2, ControlRegister3, ControlRegister4};

#[derive(Default, Debug)]
pub struct DRV8873Config {
    pub cr1: ControlRegister1,
    pub cr2: ControlRegister2,
    pub cr3: ControlRegister3,
    pub cr4: ControlRegister4,
}
