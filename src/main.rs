//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for
//! the on-board LED.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not
// need to change.
use stm32f4xx_hal as hal;
// use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use hal::{gpio::GpioExt, pac, rcc::RccExt};

#[entry]
fn main() -> ! {
  let p = pac::Peripherals::take().unwrap();

  let mut rcc = p.RCC.constrain();

  let gpioc = p.GPIOC.split(&mut rcc);
  let mut led = gpioc.pc13.into_push_pull_output();

  loop {
    info!("on!");
    for _ in 0..5_000_000 {
      led.set_high();
    }

    info!("off!");
    for _ in 0..5_000_000 {
      led.set_low();
    }
  }
}

// End of file
