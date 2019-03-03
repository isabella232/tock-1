//use crate::hil::gpio;
use crate::returncode::ReturnCode;

//type SkyResult = Result<ReturnCode, ReturnCode>;

pub trait Skyworks {
    //fn init(&mut self) -> ReturnCode;
    fn bypass(&self) -> ReturnCode;
    fn enable_pa(&self) -> ReturnCode;
    fn enable_lna(&self) -> ReturnCode;
    //fn read(&self) -> (ReturnCode, );
}

/*
pub struct CtlPin<'a> {
    pub pin: &'a mut gpio::Pin,
}

impl CtlPin<'a> {
    pub fn new(p: &'a mut gpio::Pin) -> CtlPin {
        CtlPin { pin: p }
    }
}

impl Skyworks for CtlPin<'a> {
    /*
    fn init(&mut self) {
        self.pin.make_output();
    }
    */
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
/*
fn read(&self) -> bool {
    self.pin.read()
}
*/
}
*/
