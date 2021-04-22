use stm32f7::stm32f7x2::Peripherals;
use stm32f7::stm32f7x2::{DMA1, RCC};
use stm32f7xx_hal::rcc::{RccExt, HSEClock, HSEClockMode, Clocks};
use stm32f7xx_hal::prelude::*;

use crate::synthesizer::SAMPLE_FREQ;


pub fn init_peripherals(device: &Peripherals) {
    device.RCC.ahb1enr.modify(|_, w|
        w.gpioaen().enabled()
         .gpioben().enabled()
         .gpiocen().enabled()
    );
    init_tim2(&device);
    init_dac(&device);
    device.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());
    init_leds_pwm(&device);
    init_tim1_pwm(&device);
    init_tim5_pwm(&device);
    init_tim3(&device); // Led multiplex timer
    init_tim4(&device); // Sequencer timer
    init_tim6(&device); // Input poll timer
    init_inputs(&device); // Rot enc inputs
}

pub fn init_clock(rcc: RCC) -> Clocks {
    let rcc = rcc.constrain();
    rcc.cfgr
        .hse(HSEClock::new(8.mhz(), HSEClockMode::Oscillator))
        .hclk(216.mhz())
        .pclk1(54.mhz())
        .pclk2(108.mhz())
        .sysclk(216.mhz())
        .freeze()
}

pub fn init_dac(dp: &Peripherals) {
    // enable GPIOA and DAC clocks
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.dacen().enabled());

    // configure PA04, PA05 (DAC_OUT1 & DAC_OUT2) as analog, floating
    let gpioa = &dp.GPIOA;
    gpioa.moder.modify(|_, w|
        w.moder4().analog()
         .moder5().analog());
    gpioa.pupdr.modify(|_, w|
        w.pupdr4().floating()
         .pupdr5().floating());

    // configure DAC
    let dac = &dp.DAC;
    dac.cr.modify(|_, w|
        w.boff1().disabled()     // disable dac output buffer for channel 1
         .boff2().disabled()     // disable dac output buffer for channel 2
         .ten1().enabled()       // enable trigger for channel 1
         .ten2().enabled()       // enable trigger for channel 2
         .tsel1().tim2_trgo()    // set trigger for channel 1 to TIM2
         .tsel2().tim2_trgo());  // set trigger for channel 2 to TIM2

    // enable DAC
    dac.cr.modify(|_, w|
        w.en1().enabled()    // enable dac channel 1
         .en2().enabled());  // enable dac channel 2
}

pub fn init_dma1(dma1: &DMA1, buffer: &[u32]) {
    let pa = 0x40007420; // Dual DAC 12-bit right-aligned data holding register (DHR12RD)
    let ma = buffer.as_ptr() as u32;
    let stream = &dma1.st[5];
    stream.par.modify(|_, w| w.pa().bits(pa));     // destination peripheral address
    stream.m0ar.modify(|_, w| w.m0a().bits(ma));  // source memory address
    stream.ndtr.modify(|_, w| w.ndt().bits(buffer.len() as u16));  // number of items to transfer
    stream.cr.modify(|_, w| w.chsel().bits(0b0111));  // Set channel 7 = DAC1
    stream.cr.modify(|_, w| w.pl().high());  // High priority
    stream.fcr.modify(|_, w| w.dmdis().enabled());  // Enable direct mode
    stream.cr.modify(|_, w|
        w.dir().memory_to_peripheral()
         .mburst().single()
         .minc().incremented()  // increment memory address every transfer
         .pinc().fixed()        // don't increment peripheral address every transfer
         .msize().bits32()      // memory word size is 32 bits
         .psize().bits32()      // peripheral word size is 32 bits
         .circ().enabled()      // dma mode is circular
         .teie().enabled()      // trigger an interrupt if an error occurs
         .tcie().enabled()      // trigger an interrupt when transfer is complete
         .htie().enabled()      // trigger an interrupt when half the transfer is complete
    );
    stream.cr.modify(|_, w| w.en().enabled());
}

pub fn init_leds_pwm(dp: &Peripherals) {
    dp.GPIOA.moder.modify(|_, w|
        w.moder0().alternate()
         .moder1().alternate()
         .moder2().alternate()
         .moder8().alternate()
         .moder9().alternate()
         .moder10().alternate()
    );
    dp.GPIOA.afrl.modify(|_, w|
        w.afrl0().af2()  // TIM5 CH1
         .afrl1().af2()  // TIM5 CH2
         .afrl2().af2()  // TIM5 CH3
    );
    dp.GPIOA.afrh.modify(|_, w|
        w.afrh8().af1()  // TIM1 CH1
         .afrh9().af1()  // TIM1 CH2
         .afrh10().af1()  // TIM1 CH3
    );
    
    dp.GPIOC.moder.modify(|_, w| {
        w.moder0().output()
         .moder1().output()
         .moder2().output()
         .moder3().output()
         .moder6().output()
         .moder7().output()
         .moder8().output()
         .moder9().output()
    });
}

pub fn init_inputs(dp: &Peripherals) {
    dp.GPIOA.moder.modify(|_, w|
        w.moder6().input()
         .moder7().input()
    );
    dp.GPIOA.pupdr.modify(|_, w|
        w.pupdr6().pull_up()
         .pupdr7().pull_up()
    );

    dp.GPIOB.moder.modify(|_, w|
        w.moder12().input()
         .moder13().input()
         .moder14().input()
    );
    dp.GPIOB.pupdr.modify(|_, w|
        w.pupdr12().pull_up()
         .pupdr13().pull_up()
         .pupdr14().pull_up()
    );

    dp.GPIOC.moder.modify(|_, w|
        w.moder4().input()
         .moder10().input()
         .moder11().input()
         .moder12().input()
         .moder13().input()
         .moder14().input()
         .moder15().input()
    );
    dp.GPIOC.pupdr.modify(|_, w|
        w.pupdr4().pull_up()
         .pupdr10().pull_up()
         .pupdr11().pull_up()
         .pupdr12().pull_up()
         .pupdr13().pull_up()
         .pupdr14().pull_up()
         .pupdr15().pull_up()
    );
}

pub fn init_tim1_pwm(dp: &Peripherals) {
    // PWM timer for R2, G2, B2
    // Base clock is 216 MHz
    let rcc = &dp.RCC;
    rcc.apb2enr.modify(|_, w| w.tim1en().enabled());

    let arr = 255;
    let tim1 = &dp.TIM1;

    // Setup timer
    tim1.psc.modify(|_, w| w.psc().bits(1));        // Set prescaler to 2 to set base to 54 MHz
    tim1.arr.modify(|_, w| w.arr().bits(arr));      // timer period (sysclk / fs)
    tim1.cr1.modify(|_, w| w.cen().enabled());

    // Setup channels
    tim1.ccmr1_output().modify(|_, w| w.oc1m().pwm_mode1().oc1pe().set_bit()); // Channel 1, PWM mode 1
    tim1.ccmr1_output().modify(|_, w| w.oc2m().pwm_mode1().oc2pe().set_bit()); // Channel 1, PWM mode 1
    tim1.ccmr2_output().modify(|_, w| w.oc3m().pwm_mode1().oc3pe().set_bit()); // Channel 1, PWM mode 1

    tim1.ccer.modify(|_, w| w.cc1e().set_bit());  // enable timer
    tim1.ccer.modify(|_, w| w.cc2e().set_bit());  // enable timer
    tim1.ccer.modify(|_, w| w.cc3e().set_bit());  // enable timer

    tim1.bdtr.modify(|_, w| w.moe().set_bit());
    tim1.egr.write(|w| w.ug().update());  // Update
}

pub fn init_tim5_pwm(dp: &Peripherals) {
    // PWM timer for R1, G1, B1
    // Base clock is 108 MHz
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.tim5en().enabled());

    let arr = 255;
    let tim1 = &dp.TIM5;

    // Setup timer
    tim1.arr.modify(|_, w| w.arr().bits(arr));      // timer period (sysclk / fs)
    tim1.cr1.modify(|_, w| w.cen().enabled());

    // Setup channels
    tim1.ccmr1_output().modify(|_, w| w.oc1m().pwm_mode1().oc1pe().set_bit()); // Channel 1, PWM mode 1
    tim1.ccmr1_output().modify(|_, w| w.oc2m().pwm_mode1().oc2pe().set_bit()); // Channel 1, PWM mode 1
    tim1.ccmr2_output().modify(|_, w| w.oc3m().pwm_mode1().oc3pe().set_bit()); // Channel 1, PWM mode 1

    tim1.ccer.modify(|_, w| w.cc1e().set_bit());  // enable timer
    tim1.ccer.modify(|_, w| w.cc2e().set_bit());  // enable timer
    tim1.ccer.modify(|_, w| w.cc3e().set_bit());  // enable timer

    tim1.egr.write(|w| w.ug().update());  // Update
}

pub fn init_tim2(dp: &Peripherals) {
    // DAC timer
    // Base is 108 MHz
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.tim2en().enabled());

    // calculate timer frequency
    let sysclk = 108_000_000;       // the stmf32f3 discovery board CPU runs at 8Mhz by defaBult
    let arr = sysclk / SAMPLE_FREQ;        // value to use for auto reload register (arr)

    // configure TIM2
    let tim2 = &dp.TIM2;
    tim2.cr2.modify(|_, w| w.mms().update());       // trigger interrupt when counter reaches arr value
    tim2.arr.modify(|_, w| w.arr().bits(arr));      // timer period (sysclk / fs)
    tim2.cr1.modify(|_, w| w.cen().enabled());  // enable TIM2
}

pub fn init_tim3(dp: &Peripherals) {
    // Timer for Led Multiplexing
    // Base clock is 108 MHz
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.tim3en().enabled());

    let tim3 = &dp.TIM3;
    tim3.psc.modify(|_, w| w.psc().bits(10_800-1));  // 10 KHz
    tim3.arr.modify(|_, w| w.arr().bits(10-1));  // 1 Khz
    tim3.dier.modify(|_, w| w.uie().enabled()); // enable interrupt at update
    tim3.cr1.modify(|_, w| w.cen().enabled()); 
}

pub fn init_tim4(dp: &Peripherals) {
    // Timer for sequencer
    // Base clock is 108 MHz
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.tim4en().enabled());

    let tim4 = &dp.TIM4;
    tim4.arr.modify(|_, w| w.arr().bits(1_000-1));
    tim4.psc.modify(|_, w| w.psc().bits(21_600-1));
    tim4.dier.modify(|_, w| w.uie().enabled());
    tim4.cr1.modify(|_, w| w.cen().enabled()); 
}

pub fn init_tim6(dp: &Peripherals) {
    // Timer for input polling
    // Base clock is 108 MHz
    let rcc = &dp.RCC;
    rcc.apb1enr.modify(|_, w| w.tim6en().enabled());

    let tim6 = &dp.TIM6;
    tim6.psc.modify(|_, w| w.psc().bits(10_800-1));  // 10 KHz
    tim6.arr.modify(|_, w| unsafe { w.arr().bits(10-1) });  // 1 Khz
    tim6.dier.modify(|_, w| w.uie().enabled());
    tim6.cr1.modify(|_, w| w.cen().enabled()); 
}