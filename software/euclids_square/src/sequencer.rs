use euclidean_rhythm::euclidean_rhythm;
use arrayvec::ArrayVec;
use array_init::array_init;
use itertools::izip;
use crate::leds::{LedData, layer_color};

#[derive(Debug)]
pub struct Sequencer<const NUM_SEQS: usize, const MAX_SEQLEN: usize> {
    sequences: [ArrayVec<u8, MAX_SEQLEN>; NUM_SEQS],
    steps: [usize; NUM_SEQS],
    base_led_data: LedData,
}

impl<const NUM_SEQS: usize, const MAX_SEQLEN: usize> Default for Sequencer<NUM_SEQS, MAX_SEQLEN> {
    fn default() -> Self {
        let seqs = array_init(|_| ArrayVec::new());
        let base_led_data = Self::build_base_led_data(&seqs);
        Self {
            sequences: seqs,
            steps: [0; NUM_SEQS],
            base_led_data,
        }
    }
}

impl<const NUM_SEQS: usize, const MAX_SEQLEN: usize> Sequencer<NUM_SEQS, MAX_SEQLEN> {
    pub fn set_sequence(&mut self, i: usize, len: usize, hits: usize, shift: isize) {
        let shift = shift.rem_euclid(len as isize) as usize;
        self.sequences[i] = euclidean_rhythm(hits, len);
        self.sequences[i].rotate_right(shift);
        self.steps[i] = 0;
        self.base_led_data = Self::build_base_led_data(&self.sequences);
    }

    fn build_base_led_data(sequences: &[ArrayVec<u8, MAX_SEQLEN>; NUM_SEQS]) -> LedData {
        let mut led_data = [0; 16];
        for (i, seq) in sequences.iter().enumerate() {
            for (t, _) in seq.iter().enumerate().filter(|(_, &v)| v == 1 ) {
                led_data[t] |= layer_color(i, 0x40);
            }
        }
        led_data
    }

    pub fn reset_steps(&mut self) {
        self.steps = [0; NUM_SEQS];
    }

    pub fn step(&mut self) -> ([bool; NUM_SEQS], LedData) {
        let mut gates = [false; NUM_SEQS];
        let mut led_data = self.base_led_data;

        for (i, (step, seq, gate)) in izip!(self.steps.iter_mut(), &self.sequences, gates.iter_mut()).enumerate() {
            if seq[*step] == 1 {
                *gate = true;
                led_data[*step] |= layer_color(i, 0xFF);
            }
            *gate = seq[*step] == 1;

            *step += 1;
            if *step >= seq.len() {
                *step = 0;
            }
        }

        (gates, led_data)
    }
}