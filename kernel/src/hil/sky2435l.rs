use crate::returncode::ReturnCode;

pub trait Skyworks {
    fn sleep(&self) -> ReturnCode;
    fn bypass(&self) -> ReturnCode;
    fn enable_pa(&self) -> ReturnCode;
    fn enable_lna(&self) -> ReturnCode;
}
