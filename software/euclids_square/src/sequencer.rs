use euclidean_rhythm::euclidean_rhythm;
use arrayvec::ArrayVec;
use array_init::array_init;
use itertools::izip;

#[derive(Debug)]
pub struct Sequencer<const NUM_SEQS: usize, const MAX_SEQLEN: usize> {
    pub sequences: [ArrayVec<u8, MAX_SEQLEN>; NUM_SEQS],
    pub steps: [usize; NUM_SEQS],
}

impl<const NUM_SEQS: usize, const MAX_SEQLEN: usize> Default for Sequencer<NUM_SEQS, MAX_SEQLEN> {
    fn default() -> Self {
        let seqs = array_init(|_| ArrayVec::new());
        Self {
            sequences: seqs,
            steps: [0; NUM_SEQS],
        }
    }
}

impl<const NUM_SEQS: usize, const MAX_SEQLEN: usize> Sequencer<NUM_SEQS, MAX_SEQLEN> {
    pub fn set_sequence(&mut self, i: usize, len: usize, hits: usize, shift: isize) {
        let shift = shift.rem_euclid(len as isize) as usize;
        self.sequences[i] = euclidean_rhythm(hits, len);
        self.sequences[i].rotate_right(shift);
        self.steps[i] = 0;
    }

    pub fn reset_steps(&mut self) {
        self.steps = [0; NUM_SEQS];
    }

    pub fn step(&mut self) -> [bool; NUM_SEQS] {
        let mut gates = [false; NUM_SEQS];

        for (step, seq, gate) in izip!(self.steps.iter_mut(), &self.sequences, gates.iter_mut()) {
            *gate = seq[*step] == 1;

            *step += 1;
            if *step >= seq.len() {
                *step = 0;
            }
        }
        gates
    }
}