use super::Pinmap;
use enum_primitive::cast::FromPrimitive;

pub const CHIP_ID: u32 = 0x2282f000;

enum_from_primitive!{
pub enum PIN_FN {
    UART0_RX = 12,
    UART0_TX = 13,
    I2C0_SCL = 22,
    I2C0_SDA = 5,
    TDO = 16,
    TDI = 17,
    RED_LED = 6,
    GREEN_LED = 7,
    BUTTON_1 = 15,
    BUTTON_2 = 14,
    GPIO0 = 21,
    ADC0 = 23,
    ADC1 = 24,
    ADC2 = 25,
    ADC3 = 26,
    ADC4 = 27,
    PWM0 = 18,
    PWM1 = 19,
    RF24 = 28,
    RFHIPA = 29,
    RFSUBG = 30,
}
}

pub static PINMAP: Pinmap = Pinmap {
    uart0_rx: PIN_FN::UART0_RX as usize,
    uart0_tx: PIN_FN::UART0_TX as usize,
    i2c0_scl: PIN_FN::I2C0_SCL as usize,
    i2c0_sda: PIN_FN::I2C0_SDA as usize,
    red_led: PIN_FN::RED_LED as usize,
    green_led: PIN_FN::GREEN_LED as usize,
    button1: PIN_FN::BUTTON_1 as usize,
    button2: PIN_FN::BUTTON_2 as usize,
    gpio0: PIN_FN::GPIO0 as usize,
    a0: PIN_FN::ADC0 as usize,
    a1: PIN_FN::ADC1 as usize,
    a2: PIN_FN::ADC2 as usize,
    a3: PIN_FN::ADC3 as usize,
    a4: PIN_FN::ADC4 as usize,
    a5: None,
    a6: None,
    a7: None,
    pwm0: PIN_FN::PWM0 as usize,
    pwm1: PIN_FN::PWM1 as usize,
    rf_2_4: Some(PIN_FN::RF24 as usize),
    rf_high_pa: Some(PIN_FN::RFHIPA as usize),
    rf_subg: Some(PIN_FN::RFSUBG as usize),
};
