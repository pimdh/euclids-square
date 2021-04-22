#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
extern crate stm32f7;

extern crate panic_itm;
use rtic::app;
use cortex_m::{iprintln, peripheral::ITM};
use stm32f7::stm32f7x2::Peripherals;
use stm32f7::stm32f7x2::{GPIOA, GPIOB, GPIOC, GPIOE, DMA1, DAC, TIM1, TIM3, TIM4, TIM5, TIM6};
use core::mem;

mod synthesizer;
mod sequencer;
mod leds;
mod init_peripherals;
mod inputs;

use synthesizer::{BUFFER_LEN, dma_handler, DmaState, Synth, SynthVoice};
use leds::{show_leds_pwm};
use init_peripherals::{init_peripherals, init_dma1, init_clock};
use sequencer::Sequencer;
use inputs::{Inputs, RotEvent};

const NUM_VOICES: usize = 3;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f7::stm32f7x2, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        auido_buffer: [u32; BUFFER_LEN],
        gpioa: GPIOA,
        gpiob: GPIOB,
        gpioc: GPIOC,
        gpioe: GPIOE,
        dma1: DMA1,
        dac: DAC,
        itm: ITM,
        tim1: TIM1,
        tim3: TIM3,
        tim4: TIM4,
        tim5: TIM5,
        tim6: TIM6,
        led_data: [u32; 16],
        synth: Synth<NUM_VOICES>,
        sequencer: Sequencer<NUM_VOICES, 16>,
        inputs: Inputs,
    }

    #[init(spawn = [init_dma1_task])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        let mut itm = core.ITM;
        core.DWT.enable_cycle_counter();
        let device: Peripherals = cx.device;

        init_peripherals(&device);
        init_clock(device.RCC);
        cx.spawn.init_dma1_task().unwrap();

        iprintln!(&mut itm.stim[0], "Hello, Euclid!");
        
        let synth = Synth { voices: [SynthVoice::new(0), SynthVoice::new(1), SynthVoice::new(2)]};
        let mut sequencer: Sequencer<3, 16> = Default::default();
        sequencer.set_sequence(0, 16, 4, 0);
        sequencer.set_sequence(1, 12, 7, 2);
        sequencer.set_sequence(2, 5, 3, 0);

        iprintln!(&mut itm.stim[0], "{:?}", sequencer);

        let inputs: Inputs = Default::default();

        init::LateResources {
            auido_buffer: [0; BUFFER_LEN],
            gpioa: device.GPIOA,
            gpiob: device.GPIOB,
            gpioc: device.GPIOC,
            gpioe: device.GPIOE,
            dma1: device.DMA1,
            dac: device.DAC,
            itm,
            tim1: device.TIM1,
            tim3: device.TIM3,
            tim4: device.TIM4,
            tim5: device.TIM5,
            tim6: device.TIM6,
            led_data: [0u32; 16],
            synth, 
            sequencer,
            inputs,
        }
    }

    #[task(resources=[dma1, dac, auido_buffer])]
    fn init_dma1_task(cx: init_dma1_task::Context) {
        let dma1 = cx.resources.dma1;
        let auido_buffer = cx.resources.auido_buffer;
        let dac = cx.resources.dac;
        init_dma1(dma1, auido_buffer);
        dac.cr.modify(|_, w| w.dmaen1().enabled());
    }

    #[task(binds = DMA1_STREAM5, resources = [dma1, auido_buffer, synth], priority=1)]
    fn dma1_stream5(mut cx: dma1_stream5::Context) {
        let state = dma_handler(cx.resources.dma1, &mut cx.resources.auido_buffer, &mut cx.resources.synth);
        match state {
            DmaState::Error =>  panic!("DMA error"),
            DmaState::Unknown =>  panic!("Unkonwn DMA state"),
            _ => ()
        }
    }

    // Led multiplex timer
    #[task(binds = TIM3, resources=[gpioc, tim1, tim5, led_data, tim3], priority=3)]
    fn tim3(cx: tim3::Context) {
        static mut STEP: u8 = 0;
        let gpioc = cx.resources.gpioc;
        let tim1 = cx.resources.tim1;
        let tim3 = cx.resources.tim3;
        let tim5 = cx.resources.tim5;
        let led_data = cx.resources.led_data;
        tim3.sr.modify(|_, w| w.uif().clear_bit());
        show_leds_pwm(gpioc, tim1, tim5, &led_data, *STEP);
        *STEP = (*STEP + 1) % 8;
    }

    // Sequencer timer
    #[task(binds = TIM4, resources=[sequencer, led_data, tim4, synth], priority=1)]
    fn tim4(mut cx: tim4::Context) {
        let tim4 = cx.resources.tim4;
        tim4.sr.modify(|_, w| w.uif().clear_bit());

        let (gates, new_led_data) = cx.resources.sequencer.step();
        cx.resources.led_data.lock(|led_data| {
            let _ = mem::replace(led_data, new_led_data);
        });
        cx.resources.synth.apply_gates(gates);
    }

    // Input polling
    #[task(binds = TIM6_DAC, resources=[tim6, inputs, gpioa, gpiob, gpioc, itm], priority=1)]
    fn tim6(mut cx: tim6::Context) {
        let tim6 = cx.resources.tim6;
        let inputs = cx.resources.inputs;
        tim6.sr.modify(|_, w| w.uif().clear_bit());
        let itm = cx.resources.itm;

        let gpioa_read = cx.resources.gpioa.idr.read();
        let gpiob_read = cx.resources.gpiob.idr.read();
        let gpioc_read = cx.resources.gpioc.lock(|gpioc| gpioc.idr.read());
        let event = inputs.update(gpioa_read, gpiob_read, gpioc_read);
        if let Some(v) = event.switch_a.edge {
            iprintln!(&mut itm.stim[0], "Switch A: {:#?}", v);
        }
        if let Some(v) = event.switch_b.edge {
            iprintln!(&mut itm.stim[0], "Switch B: {:#?}", v);
        }
        if let Some(v) = event.switch_c.edge {
            iprintln!(&mut itm.stim[0], "Switch C: {:#?}", v);
        }
        if let Some(v) = event.switch_d.edge {
            iprintln!(&mut itm.stim[0], "Switch D: {:#?}", v);
        }
        if event.rot_a != RotEvent::None {
            iprintln!(&mut itm.stim[0], "ROT A: {:#?}", event.rot_a);
        }
        if event.rot_b != RotEvent::None {
            iprintln!(&mut itm.stim[0], "ROT B: {:#?}", event.rot_b);
        }
        if event.rot_c != RotEvent::None {
            iprintln!(&mut itm.stim[0], "ROT C: {:#?}", event.rot_c);
        }
        if event.rot_d != RotEvent::None {
            iprintln!(&mut itm.stim[0], "ROT D: {:#?}", event.rot_d);
        }
    }

    extern "C" {
        fn EXTI1();
    }
};