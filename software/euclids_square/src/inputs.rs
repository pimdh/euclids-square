use debouncr::{debounce_stateful_2, debounce_stateful_4, DebouncerStateful, Repeat4, Repeat2, Edge};
type GpioARead = stm32f7::R<u32, stm32f7::Reg<u32, stm32f7::stm32f7x2::gpioa::_IDR>>;
type GpioBRead = stm32f7::R<u32, stm32f7::Reg<u32, stm32f7::stm32f7x2::gpiob::_IDR>>;
type GpioCRead = stm32f7::R<u32, stm32f7::Reg<u32, stm32f7::stm32f7x2::gpioh::_IDR>>;

type SwitchDebouncer = DebouncerStateful<u8, Repeat4>;
type RotDebouncer = DebouncerStateful<u8, Repeat2>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RotDirection {
    Cw,
    Ccw,
}

type RotEvent = Option<RotDirection>;


impl From<RotDirection> for isize {
    fn from(event: RotDirection) -> Self {
        match event {
            RotDirection::Cw => 1,
            RotDirection::Ccw => -1,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SwitchEvent {
    pub edge: Option<Edge>,
    pub is_high: bool,
    pub is_low: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InputEvent {
    pub switch_a: SwitchEvent,
    pub switch_b: SwitchEvent,
    pub switch_c: SwitchEvent,
    pub switch_d: SwitchEvent,
    pub rot_a: RotEvent,
    pub rot_b: RotEvent,
    pub rot_c: RotEvent,
    pub rot_d: RotEvent,
}

pub struct RotEnc {
    pin_a: RotDebouncer,
    pin_b: RotDebouncer,
}

impl Default for RotEnc {
    fn default() -> Self {
        Self {
            pin_a: debounce_stateful_2(false),
            pin_b: debounce_stateful_2(false),
        }
    }
}

impl RotEnc {
    fn update(&mut self, pin_a: bool, pin_b: bool) -> RotEvent {
        match (self.pin_a.update(pin_a), self.pin_b.update(pin_b)) {
            (Some(Edge::Rising), _) if self.pin_b.is_low() => Some(RotDirection::Cw),
            (Some(Edge::Rising), _) if self.pin_b.is_high() => Some(RotDirection::Ccw),
            _ => None
        }
    }
}

struct Switch {
    debouncer: SwitchDebouncer,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            debouncer: debounce_stateful_4(false),
        }
    }
}

impl Switch {
    fn update(&mut self, input: bool) -> SwitchEvent {
        let edge = self.debouncer.update(input);
        SwitchEvent {
            edge,
            is_high: self.debouncer.is_high(),
            is_low: self.debouncer.is_low(),
        }
    }
}

#[derive(Default)]
pub struct Inputs {
    switch_a: Switch,
    switch_b: Switch,
    switch_c: Switch,
    switch_d: Switch,
    rot_a: RotEnc,
    rot_b: RotEnc,
    rot_c: RotEnc,
    rot_d: RotEnc,
}

impl Inputs {
    pub fn update(&mut self, gpioa: GpioARead, gpiob: GpioBRead, gpioc: GpioCRead) -> InputEvent {
        InputEvent {
            switch_a: self.switch_a.update(gpioc.idr13().is_low()),
            switch_b: self.switch_b.update(gpioc.idr10().is_low()),
            switch_c: self.switch_c.update(gpioc.idr4().is_low()),
            switch_d: self.switch_d.update(gpiob.idr14().is_low()),
            rot_a: self.rot_a.update(gpioc.idr14().is_low(), gpioc.idr15().is_low()),
            rot_b: self.rot_b.update(gpioc.idr11().is_low(), gpioc.idr12().is_low()),
            rot_c: self.rot_c.update(gpioa.idr6().is_low(), gpioa.idr7().is_low()),
            rot_d: self.rot_d.update(gpiob.idr12().is_low(), gpiob.idr13().is_low()),
        }
    }
}