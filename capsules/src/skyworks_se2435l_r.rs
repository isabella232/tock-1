use kernel::hil;
use kernel::hil::rf_frontend::SE2435L;

/// Holds the array of GPIO pins attached to the control pins and implements a `Driver`
/// interface to control them.
pub struct Sky2435L<'a, G: hil::gpio::Pin> {
    cps: &'a G,
    csd: &'a G,
    ctx: &'a G,
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> Sky2435L<'a, G> {
    pub fn new(cps: &'a G, csd: &'a G, ctx: &'a G) -> Sky2435L<'a, G> {
        Sky2435L { cps, csd, ctx }
    }
}

impl<G: hil::gpio::Pin + hil::gpio::PinCtl> SE2435L for Sky2435L<'a, G> {
    fn sleep(&self) -> kernel::ReturnCode {
        self.cps.clear();
        self.csd.clear();
        self.ctx.clear();
        kernel::ReturnCode::SUCCESS
    }

    fn bypass(&self) -> kernel::ReturnCode {
        self.cps.clear();
        self.csd.set();
        self.ctx.clear();
        kernel::ReturnCode::SUCCESS
    }

    fn enable_pa(&self) -> kernel::ReturnCode {
        // cps is ignored
        self.csd.set();
        self.ctx.set();
        kernel::ReturnCode::SUCCESS
    }

    fn enable_lna(&self) -> kernel::ReturnCode {
        self.cps.set();
        self.csd.set();
        self.ctx.clear();
        kernel::ReturnCode::SUCCESS
    }
}
