use stm32f4xx_hal::rcc::{Enable, Reset};

use crate::hal::pac;

const FRAME_BITS: usize = 24 * 50;
const RESET_SLOTS: usize = 64;
const DMA_WORDS: usize = FRAME_BITS + RESET_SLOTS;
const ARR: u32 = 104;
const DUTY_0: u32 = (ARR + 1) / 3;
const DUTY_1: u32 = (ARR + 1) * 2 / 3;

static mut WS_BITS: [u32; DMA_WORDS] = [0; _];

pub struct LedStrip<const N: usize> {
  buffer: [u32; 24],
}

impl<const N: usize> LedStrip<N> {
  pub fn new(rcc: &mut pac::RCC, timer: &pac::TIM2, dma: &pac::DMA1) -> Self {
    pac::TIM2::enable(rcc);
    pac::TIM2::reset(rcc);
    pac::DMA1::enable(rcc);
    pac::DMA1::reset(rcc);

    unsafe {
      for i in 0..FRAME_BITS {
        WS_BITS[i] = DUTY_0;
      }

      // WS_BITS[0] = DUTY_1;
    }

    timer.psc().write(|w| unsafe { w.psc().bits(0) });
    timer.arr().write(|w| unsafe { w.arr().bits(ARR) });
    timer.ccr1().write(|w| unsafe { w.ccr().bits(0) });
    timer.ccmr1_output().modify(|_, w| {
      w.oc1pe().set_bit();
      w.oc1m().pwm_mode1()
    });
    timer.ccer().modify(|_, w| w.cc1e().set_bit());
    timer.dier().modify(|_, w| w.cc1de().set_bit());
    timer.egr().write(|w| w.ug().set_bit());
    timer.cr1().modify(|_, w| w.arpe().set_bit().cen().set_bit());

    dma.st(5).cr().modify(|_, w| w.en().clear_bit());
    while dma.st(5).cr().read().en().bit_is_set() {}

    dma
      .hifcr()
      .write(|w| w.ctcif5().set_bit().chtif5().set_bit().cdmeif5().set_bit().cfeif5().set_bit());

    dma.st(5).par().write(|w| unsafe { w.bits(timer.ccr1().as_ptr() as u32) });
    dma.st(5).m0ar().write(|w| unsafe { w.bits(core::ptr::addr_of!(WS_BITS) as u32) });
    dma.st(5).ndtr().write(|w| unsafe { w.ndt().bits(DMA_WORDS as u16) });
    dma.st(5).cr().write(|w| unsafe {
      w.chsel()
        .bits(3)
        .dir()
        .memory_to_peripheral()
        .minc()
        .set_bit()
        .pinc()
        .clear_bit()
        .msize()
        .bits32()
        .psize()
        .bits32()
        .circ()
        .set_bit()
        .pl()
        .very_high()
        .en()
        .set_bit()
    });

    LedStrip { buffer: [0; _] }
  }
}
