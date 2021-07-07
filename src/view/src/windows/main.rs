mod header;

use std::sync::{Arc, Mutex};

use gui::elements::Element;

use crate::{
    model::{CakeModel, CakeViewModel},
    windows::main::header::MainWindowHeader,
};

pub struct MainWindowElement {
    flex: Box<dyn Element<CakeViewModel>>,
}

impl MainWindowElement {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Self {
        use gui::{d, elements::FlexElement, rgb, rgba, size, style};
        use stretch::style::{AlignItems, FlexDirection};

        let model = model.clone();

        let flex = FlexElement::new(
            rgba!(0, 0, 0, 0),
            style!(size => size!(100, %; 100, %), flex_direction => FlexDirection::Column, align_items => AlignItems::Stretch),
            vec![MainWindowHeader::new(&model)],
        );

        MainWindowElement { flex }
    }
}

impl Element<CakeViewModel> for MainWindowElement {
    fn layout(
        &mut self,
        stretch: &mut stretch::Stretch,
        model: &mut CakeViewModel,
    ) -> Result<stretch::node::Node, stretch::Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, stretch: &stretch::Stretch, ui: &imgui::Ui, model: &mut CakeViewModel) {
        self.flex.render(stretch, ui, model)
    }
}
