use kernel::hil;
use kernel::hil::sky2435l::Skyworks;
use kernel::{AppId, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::SKY2435L as usize;

type SkyResult = Result<ReturnCode, ReturnCode>;

/// Whether the SKY control pins are active high or active low on this platform.
#[derive(Clone, Copy)]
pub enum ActivationMode {
    ActiveHigh,
    ActiveLow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CtlPinType {
    Cps = 0,
    Csd = 1,
    Ctx = 2,
}

/// Holds the array of GPIO pins attached to the control pins and implements a `Driver`
/// interface to control them.
pub struct Sky2435L<'a, G: hil::gpio::Pin> {
    pins_init: &'a [(&'a G, ActivationMode, CtlPinType)],
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Sky2435L<'a, G> {
    pub fn new(pins_init: &'a [(&'a G, ActivationMode, CtlPinType)]) -> Sky2435L<'a, G> {
        // Make all pins output and off
        for &(pin, mode, _pin_type) in pins_init.as_ref().iter() {
            pin.make_output();
            match mode {
                ActivationMode::ActiveHigh => pin.clear(),
                ActivationMode::ActiveLow => pin.set(),
            }
        }

        Sky2435L {
            pins_init: pins_init,
        }
    }

    // PIN CONTROLS FOR INTERNAL CALLS
    fn set_pin(&self, pin_type: CtlPinType) -> SkyResult {
        let pins_init = self.pins_init.as_ref();
        for pin_data in pins_init {
            let (pin, mode, p_type) = pin_data;
            if *p_type == pin_type {
                match mode {
                    ActivationMode::ActiveHigh => pin.set(),
                    ActivationMode::ActiveLow => pin.clear(),
                }
                //debug!("pin: {:?} set", pin_type);
                return Ok(ReturnCode::SUCCESS);
            }
        }
        debug!("no {:?} set pin found wtf?", pin_type);
        return Err(ReturnCode::FAIL); // Should never happen if set up correctly.
    }

    fn clear_pin(&self, pin_type: CtlPinType) -> SkyResult {
        let pins_init = self.pins_init.as_ref();
        for pin_data in pins_init {
            let (pin, mode, p_type) = pin_data;
            if *p_type == pin_type {
                match mode {
                    ActivationMode::ActiveHigh => pin.clear(),
                    ActivationMode::ActiveLow => pin.set(),
                }
                //debug!("pin: {:?} clear", pin_type);
                return Ok(ReturnCode::SUCCESS);
            }
        }
        debug!("no {:?} clear pin found wtf?", pin_type);
        return Err(ReturnCode::FAIL); // Should never happen if set up correctly.
    }
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Driver for Sky2435L<'a, G> {
    /// Control the Sky2435L pins.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Returns SUCCESS if there is a Skyworks chip available on the board
    /// - `1`: Set the SKY to enabled. Returns `EINVAL` if chip is not available.
    /// - `2`: Set the SKY to diabled. Returns `EINVAL` if chip is not available.
    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        let pins_init = self.pins_init.as_ref();
        match command_num {
            // is skyworks chip available?
            0 => ReturnCode::SuccessWithValue {
                value: pins_init.len() as usize,
            },

            // on
            1 => ReturnCode::SUCCESS,

            // off
            2 => ReturnCode::SUCCESS,

            // bypass
            3 => ReturnCode::SUCCESS,
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

// The truth table for the Skyworks SE2435L can be found at
// http://www.skyworksinc.com/uploads/documents/SE2435L_202412I.pdf
//
// The matrix reads as follows:
//       MODE       |  CPS  |  CSD  | CTX  |
// Sleep            |  Low  |  Low  | Low  |
//-----------------------------------------|
// Rx or Tx bypass  |  Low  |  High | Low  |
// ________________________________________|
// Rx LNA mode      |  High |  High | Low  |
// ________________________________________|
// Transmit         |  N/A  |  High | High |
// ________________________________________|
//
impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Skyworks for Sky2435L<'a, G> {
    // Bypass mode pin settings: CPS: Low, CSD: Low, CTX: Low
    fn bypass(&self) -> ReturnCode {
        self.clear_pin(CtlPinType::Cps)
            .and_then(|_| self.clear_pin(CtlPinType::Csd))
            .and_then(|_| self.clear_pin(CtlPinType::Ctx))
            .unwrap_or(ReturnCode::FAIL)
    }
    // PA mode pin settings: CPS: N/A, CSD: High, CTX: High
    fn enable_pa(&self) -> ReturnCode {
        self.clear_pin(CtlPinType::Cps)
            .and_then(|_| self.set_pin(CtlPinType::Csd))
            .and_then(|_| self.set_pin(CtlPinType::Ctx))
            .unwrap_or(ReturnCode::FAIL)
    }

    // LNA mode pin settings: CPS: High, CSD: High, CTX:Low
    fn enable_lna(&self) -> ReturnCode {
        self.set_pin(CtlPinType::Cps)
            .and_then(|_| self.set_pin(CtlPinType::Csd))
            .and_then(|_| self.clear_pin(CtlPinType::Ctx))
            .unwrap_or(ReturnCode::FAIL)
    }
}
