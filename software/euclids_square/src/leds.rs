use stm32f7::stm32f7x2::{GPIOC, TIM1, TIM5};

pub fn show_leds_pwm(gpioc: &GPIOC, tim1: &TIM1, tim5: &TIM5, data: &[u32; 16], step: u8) {
    gpioc.odr.modify(|_, w| {
        w.odr0().low()
         .odr1().low()
         .odr2().low()
         .odr3().low()
         .odr6().low()
         .odr7().low()
         .odr8().low()
         .odr9().low()
    });

    let (l1, l2) = match step {
        0 => (0, 7),
        1 => (1, 6),
        2 => (2, 5),
        3 => (3, 4),
        4 => (15, 8),
        5 => (14, 9),
        6 => (13, 10),
        7 => (12, 11),
        _ => panic!()
    };

    tim5.ccr3.modify(|_, w| w.ccr().bits((data[l1] >> 16 & 0xFF) as u32));   // TIM5 CH3 = R1
    tim5.ccr2.modify(|_, w| w.ccr().bits((data[l1] >> 8 & 0xFF) as u32 / 2)); // TIM5 CH2 = G1
    tim5.ccr1.modify(|_, w| w.ccr().bits((data[l1] & 0xFF) as u32 / 2)); // TIM5 CH1 = B1
    tim1.ccr1.modify(|_, w| w.ccr().bits((data[l2] >> 16 & 0xFF) as u16));   // TIM1 CH1 = R2
    tim1.ccr2.modify(|_, w| w.ccr().bits((data[l2] >> 8 & 0xFF) as u16 / 2)); // TIM1 CH1 = G2
    tim1.ccr3.modify(|_, w| w.ccr().bits((data[l2] & 0xFF) as u16 / 2)); // TIM1 CH1 = B2

    tim1.egr.write(|w| w.ug().update());  // Update
    tim5.egr.write(|w| w.ug().update());  // Update

    gpioc.odr.modify(|_, w| {
        w.odr0().bit(step == 0)
         .odr1().bit(step == 1)
         .odr2().bit(step == 2)
         .odr3().bit(step == 3)
         .odr6().bit(step == 4)
         .odr7().bit(step == 5)
         .odr8().bit(step == 6)
         .odr9().bit(step == 7)
    });
}