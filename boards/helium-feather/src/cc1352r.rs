use super::Pinmap;
use crate::enum_primitive::cast::FromPrimitive;
use crate::enum_primitive::enum_from_primitive;

enum_from_primitive! {
pub enum PIN_FN {
    I2C0_SDA = 5,
    RED_LED = 6,
    GREEN_LED = 7,
    SPI_MISO = 8,
    SPI_MOSI = 9,
    SPI_CLK = 10,
    SPI_CS = 11,
    UART1_RX = 12,
    UART1_TX = 13,
    BUTTON_2 = 14,
    BUTTON_1 = 15,
    TDO = 16,
    TDI = 17,
    GPIO0 = 18,
    ON2 = 19,
    USB_UART0_RX = 20,
    USB_UART0_TX = 21,
    I2C0_SCL = 22,
    ADC0 = 23,
    ADC1 = 24,
    VBAT_SNS = 25,
    ISL_INT = 26,
    BMI_INT = 27,
    SE_CSD = 28,
    SE_CPS = 29,
    SE_CTX = 30,
}
}

pub static PINMAP: Pinmap = Pinmap {
    uart0_rx: PIN_FN::USB_UART0_RX as usize,
    uart0_tx: PIN_FN::USB_UART0_TX as usize,
    uart1_rx: PIN_FN::UART1_RX as usize,
    uart1_tx: PIN_FN::UART1_TX as usize,
    i2c0_scl: PIN_FN::I2C0_SCL as usize,
    i2c0_sda: PIN_FN::I2C0_SDA as usize,
    red_led: PIN_FN::RED_LED as usize,
    green_led: PIN_FN::GREEN_LED as usize,
    button1: PIN_FN::BUTTON_1 as usize,
    button2: PIN_FN::BUTTON_2 as usize,
    on2: PIN_FN::ON2 as usize,
    skyworks_csd: PIN_FN::SE_CSD as usize,
    skyworks_cps: PIN_FN::SE_CPS as usize,
    skyworks_ctx: PIN_FN::SE_CTX as usize,
    rf_2_4: None,
    rf_high_pa: None,
    rf_subg: None,
};
