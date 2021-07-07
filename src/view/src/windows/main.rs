mod header;
mod keyboard;
mod midi;

use std::sync::{Arc, Mutex};

use gui::elements::Element;

use crate::{
    model::CakeModel,
    windows::main::{header::MainWindowHeader, keyboard::MainWindowKeyboard, midi::MainWindowMidi},
};

pub struct MainWindowElement {
    flex: Box<dyn Element<CakeModel>>,
}

impl MainWindowElement {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Self {
        use gui::{d, elements::FlexColorElement, rgba, size, style};
        use stretch::style::{AlignItems, FlexDirection};

        let model = model.clone();

        let flex = FlexColorElement::new(
            // palette!(bg),
            rgba!(0, 0, 0, 0),
            style!(size => size!(100, %; 100, %), flex_direction => FlexDirection::Column, align_items => AlignItems::Stretch),
            vec![
                MainWindowHeader::new(&model),
                MainWindowMidi::new(&model),
                MainWindowKeyboard::new(&model),
            ],
        );

        Self { flex }
    }
}

impl Element<CakeModel> for MainWindowElement {
    fn layout(
        &mut self,
        stretch: &mut stretch::Stretch,
        model: &mut CakeModel,
    ) -> Result<stretch::node::Node, stretch::Error> {
        self.flex.layout(stretch, model)
    }

    fn render(
        &mut self,
        anchor: [f32; 2],
        stretch: &stretch::Stretch,
        ui: &imgui::Ui,
        model: &mut CakeModel,
    ) {
        self.flex.render(anchor, stretch, ui, model)
    }
}
