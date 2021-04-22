use stm32f7::stm32f7x2::{DMA1};

pub const SAMPLE_FREQ: u32 = 44_100;

pub const BUFFER_LEN: usize = 1024;  // Half buffer filled at around 100Hz

static KICK: &[u8; 13230] = include_bytes!("../sounds/kick.pcm");
static SNARE: &[u8; 19200] = include_bytes!("../sounds/snare.pcm");
static HIHAT: &[u8; 4410] = include_bytes!("../sounds/hihat.pcm");

#[derive(Debug)]
pub enum DmaState { Ht, Tc, Error, Unknown }


static SOUND_STORE: [&[u8]; 3] = [KICK, SNARE, HIHAT];

pub struct SynthVoice {
    pub sound: usize,
    pub volume: f32,
    step: usize,
    playing: bool,
}

impl SynthVoice {
    pub fn new(sound: usize) -> Self {
        SynthVoice {
            sound,
            volume: 1.,
            step: 0,
            playing: false,
        }
    }

    fn apply_gate(&mut self, gate: bool) {
        if gate {
            self.step = 0;
            self.playing = true;
        }
    }

    fn step(&mut self) -> Option<f32> {
        let sound = SOUND_STORE[self.sound];
        if self.playing {
            if self.step < sound.len() {
                let val = (sound[self.step] as f32) / 128. - 1.;
                self.step += 1;
                Some(val * self.volume)
            } else {
                self.playing = false;
                None
            }
        } else {
            None
        }
    }
}

pub struct Synth<const NUM_VOICES: usize> {
    pub voices: [SynthVoice; NUM_VOICES],
}

impl<const NUM_VOICES: usize> Synth<NUM_VOICES> {
    pub fn apply_gates(&mut self, gates: [bool; NUM_VOICES]) {
        for (voice, &gate) in self.voices.iter_mut().zip(gates.iter()) {
            voice.apply_gate(gate);
        }
    }

    fn step(&mut self) -> f32 {
        let mut v = 0.;
        for voice in self.voices.iter_mut() {
            v += voice.step().unwrap_or(0.);
        }
        if v > 1. {
            v = 1.;
        } else if v < -1. {
            v = -1.;
        }
        v
    }
}

pub fn dma_handler<const NUM_VOICES: usize>(dma1: &DMA1, buffer: &mut [u32; BUFFER_LEN], synth: &mut Synth<NUM_VOICES>) -> DmaState {
    let mid = buffer.len() / 2;

    let isr = dma1.hisr.read();
    let state = if isr.tcif5().is_complete() {
        dma1.hifcr.write(|w| w.ctcif5().clear());
        DmaState::Tc
    } else if isr.htif5().is_half() {
        dma1.hifcr.write(|w| w.chtif5().clear());
        DmaState::Ht
    } else if isr.teif5().is_error() {
        dma1.hifcr.write(|w| w.cteif5().clear());
        DmaState::Error
    } else {
        DmaState::Unknown
    };

    match state {
        DmaState::Ht => synth_callback(&mut buffer[0..mid], synth),
        DmaState::Tc => synth_callback(&mut buffer[mid..], synth),
        _ => (),
    }
    state
}

fn synth_callback<const NUM_VOICES: usize>(buffer: &mut [u32], synth: &mut Synth<NUM_VOICES>) {
    for val in buffer.iter_mut() {
        let v = synth.step();
        let v_12bit = ((v + 1.) * 2047.5) as u32;

        let channel_1 = v_12bit;
        let channel_2 = v_12bit;
        *val = (channel_2 << 16) + channel_1;
    }
}