use core::cmp;
use arrayvec::ArrayVec;
use array_init::array_init;
use debouncr::Edge;
use crate::inputs::InputEvent;
use crate::leds::LedData;

#[derive(Debug, Clone)]
pub struct UiState<const NUM_LAYERS: usize> {
    is_playing: bool,
    tempo: usize,
    active_layer: usize,
    time_since_last_action: usize,
    layers: [LayerState; NUM_LAYERS],
    view: View,
}

#[derive(Debug, Clone)]
enum View {
    Player,
    Sequencer,
    Sound,
    Tempo,
    Volume,
}

#[derive(Debug, Clone)]
pub struct LayerState {
    pub sound: usize,
    pub volume: usize,
    pub length: usize,
    pub hits: usize,
    pub shift: isize,
}

impl<const NUM_LAYERS: usize> Default for UiState<NUM_LAYERS> {
    fn default() -> Self {
        Self {
            is_playing: true,
            tempo: 8,
            active_layer: 0,
            time_since_last_action: 0,
            layers: array_init(|i| LayerState {
                sound: i,
                volume: 8,
                length: 16,
                hits: (if i == 0 { 1 } else { 0 }),
                shift: 0,
            }),
            view: View::Player,
        }
    }
}

pub enum OutputEvent {
    LayerUpdate (usize, LayerState),
    IsPlaying (bool),
    TempoUpdate (usize),
}

fn clamp<T: Ord>(v: T, min: T, max: T) -> T {
    cmp::max(min, cmp::min(v, max))
}

impl<const NUM_LAYERS: usize> UiState<NUM_LAYERS> {
    pub fn update(&mut self, input: InputEvent) -> ArrayVec<OutputEvent, 8> {
        let mut output_events = ArrayVec::new();

        // Switch layer
        if input.switch_c.edge == Some(Edge::Rising) {
            self.active_layer = (self.active_layer+1) % NUM_LAYERS;
            self.view = View::Sequencer;
            self.time_since_last_action = 0;
        }

        // Play / pause
        if input.switch_b.edge == Some(Edge::Rising) {
            self.is_playing = !self.is_playing;
            output_events.push(OutputEvent::IsPlaying(self.is_playing));
        }

        // Tempo
        if let Some(dir) = input.rot_b {
            self.tempo = clamp(self.tempo as isize + isize::from(dir), 1, 16) as usize;
            self.view = View::Tempo;
            self.time_since_last_action = 0;
            output_events.push(OutputEvent::IsPlaying(self.is_playing));
        }

        // Sound
        if let Some(_dir) = input.rot_b {
            // TODO
        }

        // Sequencer actions
        if input.rot_a.is_some() || input.rot_c.is_some() || input.rot_d.is_some() {
            let layer = &mut self.layers[self.active_layer];
            // Shift
            if let Some(dir) = input.rot_a {
                layer.shift += isize::from(dir);
            }

            // Length
            if let Some(dir) = input.rot_c {
                let len = layer.length as isize + isize::from(dir);
                layer.length = clamp(len, 1, 16) as usize;
                layer.hits = cmp::min(layer.hits, layer.length);
            }
            
            // Hits
            if let Some(dir) = input.rot_d {
                let hits = layer.hits as isize + isize::from(dir);
                let hits = clamp(hits, 0, layer.length as isize) as usize;
                layer.hits = hits;
            }
            self.view = View::Sequencer;
            self.time_since_last_action = 0;
            output_events.push(OutputEvent::LayerUpdate(self.active_layer, layer.clone()));
        }

        // Back to Player after inaction
        if self.time_since_last_action > 3000 {
            self.view = View::Player;
            self.time_since_last_action = 0;
        }

        output_events
    }

    pub fn render(&self, sequencer_led_data: &LedData) -> LedData {
        *sequencer_led_data
    }
}