#[derive(core::fmt::Debug)]
pub enum InputMode<PW: SetDutyCycle, P: StatefulOutputPin> {
    PhaseEnable(Option<(PW, P)>),
    PWM(Option<(PW, PW)>),
    IndependentHalfBridge(Option<(PW, PW)>),
    InputDisabled,
}

impl<PW: SetDutyCycle, P: StatefulOutputPin> InputMode<PW, P> {
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
}
