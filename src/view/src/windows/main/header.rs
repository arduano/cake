use std::sync::{Arc, Mutex};

use gui::elements::Element;
use imgui::ImColor32;

use crate::model::{CakeModel, CakeViewModel};

pub struct MainWindowHeader {
    flex: Box<dyn Element<CakeViewModel>>,
}

impl MainWindowHeader {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Box<Self> {
        use gui::{
            d,
            elements::{FlexElement, RippleButton, ToggleButton},
            rgb, rgba, size, style,
        };
        use stretch::style::{AlignItems, FlexDirection};

        let model = model.clone();

        let flex = FlexElement::new(
            rgba!(0, 0, 0, 0),
            style!(size => size!(100, %; 100, px), flex_direction => FlexDirection::Column, align_items => AlignItems::Stretch),
            vec![
                // Row 1
                FlexElement::new(
                    rgba!(0, 0, 0, 70),
                    style!(size => size!(100, %; auto)),
                    vec![
                        ToggleButton::new(
                            rgba!(0, 0, 0, 0),
                            rgba!(255, 255, 255, 100),
                            style!(size => size!(40, px; 40, px)),
                            move |model: &mut CakeViewModel| -> bool { model.paused },
                            move |model: &mut CakeViewModel| {
                                model.paused = true;
                            },
                            vec![],
                        ),
                        ToggleButton::new(
                            rgba!(0, 0, 0, 0),
                            rgba!(255, 255, 255, 100),
                            style!(size => size!(40, px; 40, px)),
                            move |model: &mut CakeViewModel| -> bool { !model.paused },
                            move |model: &mut CakeViewModel| {
                                model.paused = false;
                            },
                            vec![],
                        ),
                    ],
                ),
                // Row 2
                FlexElement::new(
                    rgba!(0, 0, 0, 0),
                    style!(size => size!(100, %; auto)),
                    vec![],
                ),
            ],
        );

        Box::new(MainWindowHeader { flex })
    }
}

impl Element<CakeViewModel> for MainWindowHeader {
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
