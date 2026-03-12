//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for
//! the on-board LED.
#![no_std]
#![no_main]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use fugit::RateExtU32;

use fl_sim::{
  Simulation,
  nalgebra::{self, point, vector},
};

use defmt::info;

mod led;

const WORLD_WIDTH: usize = 8;
const WORLD_HEIGHT: usize = 8;
const PARTICLES: usize = 128;
const _: () = assert!(WORLD_WIDTH * WORLD_HEIGHT <= u8::MAX as usize, "size is too large");

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not
// need to change.
use stm32f4xx_hal as hal;
// use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use hal::{gpio::GpioExt, pac, rcc::RccExt};

use embedded_alloc::LlffHeap as Heap;

use crate::led::{LedStrip, Pixel};

#[global_allocator]
static HEAP: Heap = Heap::empty();

fn make_simulation() -> Simulation<PARTICLES> {
  let mut simulation = Simulation::new(vector![WORLD_WIDTH as f32, WORLD_HEIGHT as f32]);

  let mut i = 0;
  for y in 4..4 + 8 {
    for x in 4..4 + 16 {
      simulation.set_particle(i, point![x as f32 / 4.0, y as f32 / 4.0]);
      i += 1;
    }
  }

  // simulation.add_barrier(point![4.0, 1.0], point![5.0, 2.0]);

  simulation
}

extern crate alloc;

#[entry]
fn main() -> ! {
  let p = pac::Peripherals::take().unwrap();
  let mut rcc = p.RCC.freeze(hal::rcc::Config::hsi().sysclk(84.MHz()));

  let gpioa = p.GPIOA.split(&mut rcc);

  let mut leds = LedStrip::new(&mut rcc, gpioa.pa0, &p.TIM2, &p.DMA1);

  const LEN: usize = 144;
  const PIX: Pixel = Pixel { r: 0, g: 0, b: 8 };

  loop {
    for i in 0..LEN {
      for i in 0..LEN {
        leds.set(i, Pixel::BLACK);
      }
      leds.set(i, PIX);

      delay(500000);
    }

    for i in (0..LEN).rev() {
      for i in 0..LEN {
        leds.set(i, Pixel::BLACK);
      }
      leds.set(i, PIX);

      delay(500000);
    }

    cortex_m::asm::nop();
  }
}

fn main_2() -> ! {
  unsafe {
    embedded_alloc::init!(HEAP, 1024 * 32);
  }

  let p = pac::Peripherals::take().unwrap();
  let mut c = cortex_m::peripheral::Peripherals::take().unwrap();
  c.DCB.enable_trace();
  c.DWT.set_cycle_count(0);
  c.DWT.enable_cycle_counter();

  let mut rcc = p.RCC.freeze(hal::rcc::Config::hsi().sysclk(84.MHz()));

  let gpioc = p.GPIOC.split(&mut rcc);
  let _led = gpioc.pc13.into_push_pull_output();

  defmt::info!("cpu frequency: {} Hz", rcc.clocks.sysclk().raw());
  let mut sim = make_simulation();

  let mut density = [[0.0; WORLD_WIDTH]; WORLD_HEIGHT];
  let mut counts = [[0u32; WORLD_WIDTH]; WORLD_HEIGHT];

  defmt::info!("cells: {}", sim.index.cell_count());
  defmt::info!("size of vec: {}", core::mem::size_of::<alloc::vec::Vec<u32>>());
  defmt::info!("w: {}, h: {}", sim.index.width(), sim.index.height());

  let mut cycle = cortex_m::peripheral::DWT::cycle_count();

  loop {
    defmt::info!("tick start at {}", cycle);
    sim.tick();

    let new_cycle = cortex_m::peripheral::DWT::cycle_count();
    let delta = new_cycle - cycle;
    defmt::info!("tick end, delta: {}", delta);
    cycle = new_cycle;

    for it in density.iter_mut().flatten() {
      *it = 0.0;
    }
    for it in counts.iter_mut().flatten() {
      *it = 0;
    }
    for p in sim.particles() {
      let y = p.position.y.clamp(0.0, WORLD_HEIGHT as f32 - 1.0) as usize;
      let x = p.position.x.clamp(0.0, WORLD_WIDTH as f32 - 1.0) as usize;
      density[y][x] += p.density;
      counts[y][x] += 1;
    }
    for y in 0..WORLD_HEIGHT {
      for x in 0..WORLD_WIDTH {
        if counts[y][x] > 0 {
          density[y][x] /= counts[y][x] as f32;
        }
      }
    }

    for y in (0..WORLD_HEIGHT).rev() {
      let mut row = [b' '; WORLD_WIDTH];
      for x in 0..WORLD_WIDTH {
        let d = density[y][x];
        if d >= 0.1 {
          row[x] = b'#';
        }
      }
      let row = core::str::from_utf8(&row).unwrap();
      defmt::println!("{}", row);
    }

    let separator = core::str::from_utf8(&[b'='; WORLD_WIDTH]).unwrap();
    defmt::println!("{}", separator);
  }
}

// End of file
