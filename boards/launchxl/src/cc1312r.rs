use super::Pinmap;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

#[allow(dead_code)]
pub const CHIP_ID: u32 = 0x20828000;

enum_from_primitive!{
pub enum PIN_FN {
    UART0_RX = 2,
    UART0_TX = 3,
    UART1_RX = 12,
    UART1_TX = 11,
    I2C0_SCL = 4,
    I2C0_SDA = 5,
    TDO = 16,
    TDI = 17,
    RED_LED = 6,
    GREEN_LED = 7,
    BUTTON_1 = 13,
    BUTTON_2 = 14,
    GPIO0 = 22,
    ADC0 = 23,
    ADC1 = 24,
    ADC2 = 25,
    ADC3 = 26,
    ADC4 = 27,
    ADC5 = 28,
    ADC6 = 29,
    ADC7 = 30,
    PWM0 = 18,
    PWM1 = 19,
}
}

pub static PINMAP: Pinmap = Pinmap {
    uart0_rx: PIN_FN::UART0_RX as usize,
    uart0_tx: PIN_FN::UART0_TX as usize,
    uart1_rx: PIN_FN::UART1_RX as usize,
    uart1_tx: PIN_FN::UART1_TX as usize,
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
    a5: Some(PIN_FN::ADC5 as usize),
    a6: Some(PIN_FN::ADC6 as usize),
    a7: Some(PIN_FN::ADC7 as usize),
    pwm0: PIN_FN::PWM0 as usize,
    pwm1: PIN_FN::PWM1 as usize,
    rf_2_4: None,
    rf_high_pa: None,
    rf_subg: None,
};

// Booster pack standard pinout
//
// 1  -> 3v3
// 2  -> DIO23 (analog)
// 3  -> DIO3  (UARTRX)
// 4  -> DIO2  (UARTTX)
// 5  -> DIO22 (GPIO)
// 6  -> DIO24 (analog)
// 7  -> DIO10 (SPI CLK)
// 8  -> DIO21 (GPIO)
// 9  -> DIO4  (I2CSCL)
// 10 -> DIO5  (I2CSDA)
//
// 11 -> DIO15 (GPIO)
// 12 -> DIO14 (SPI CS - other)
// 13 -> DIO13 (SPI CS - display)
// 14 -> DIO8  (SPI MISO)
// 15 -> DIO9  (SPI MOSI)
// 16 -> LPRST
// 17 -> unused
// 18 -> DIO11 (SPI CS - RF)
// 19 -> DIO12 (PWM)
// 20 -> GND
//
// 21 -> 5v
// 22 -> GND
// 23 -> DIO25 (analog)
// 24 -> DIO26 (analog)
// 25 -> DIO17 (analog)
// 26 -> DIO28 (analog)
// 27 -> DIO29 (analog)
// 28 -> DIO30 (analog)
// 29 -> DIO0  (GPIO)
// 30 -> DIO1  (GPIO)
//
// 31 -> DIO17
// 32 -> DIO16
// 33 -> TMS
// 34 -> TCK
// 35 -> BPRST
// 36 -> DIO18 (PWM)
// 37 -> DIO19 (PWM)
// 38 -> DIO20 (PWM)
// 39 -> DIO6  (PWM)
// 40 -> DIO7  (PWM)
