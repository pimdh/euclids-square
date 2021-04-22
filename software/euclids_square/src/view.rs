use crate::ui::{UiState, LayerState, ViewState};
use crate::sequencer::Sequencer;
use crate::leds::LedData;

pub fn render<const NUM_LAYERS: usize, const MAX_SEQLEN: usize>(ui: &UiState<NUM_LAYERS>, sequencer: &Sequencer<NUM_LAYERS, MAX_SEQLEN>) -> LedData {
    match ui.view {
        ViewState::Sequencer => render_sequencer(ui.active_layer, sequencer),
        _ => render_player(sequencer),
    }
}

pub fn layer_color(i: usize, val: u8) -> u32 {
    (val as u32) << ((i % 3) * 8)
}

fn render_player<const NUM_LAYERS: usize, const MAX_SEQLEN: usize>(sequencer: &Sequencer<NUM_LAYERS, MAX_SEQLEN>) -> LedData {
    let mut led_data = [0; 16];
    for (i, seq) in sequencer.sequences.iter().enumerate() {
        for (t, &v) in seq.iter().enumerate() {
            if v == 1 {
                led_data[t] |= layer_color(i, 0x40);
            }
        }
    }
    for (i, (&step, seq)) in sequencer.steps.iter().zip(&sequencer.sequences).enumerate() {
        if seq[step] == 1 {
            led_data[step] |= layer_color(i, 0xFF)
        }
    }
    led_data
}

fn render_sequencer<const NUM_LAYERS: usize, const MAX_SEQLEN: usize>(active_layer: usize, sequencer: &Sequencer<NUM_LAYERS, MAX_SEQLEN>) -> LedData {
    let mut led_data = [0; 16];
    let seq = &sequencer.sequences[active_layer];
    for (t, &v) in seq.iter().enumerate() {
        if v == 1 {
            led_data[t] |= layer_color(active_layer, 0xFF);
        } else {
            led_data[t] |= layer_color(active_layer, 0x10);

        }
    }
    led_data
}