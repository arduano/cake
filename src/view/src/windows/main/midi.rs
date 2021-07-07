use std::sync::{Arc, Mutex};

use gui::elements::Element;

use crate::{model::CakeModel, palette};

pub struct MainWindowMidi {
    flex: Box<dyn Element<CakeModel>>,
}

impl MainWindowMidi {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Box<Self> {
        use gui::{
            d,
            elements::{FlexColorElement, FlexElement, FlexMultiColorElement},
            rect, rgba, rgbaf, size, style,
        };
        use stretch::style::{AlignItems, FlexDirection, PositionType};

        let model = model.clone();

        let flex = FlexColorElement::new(
            rgba!(0, 0, 0, 0),
            style!(size => size!(100, %; 100, %), flex_basis => d!(100, %), position_type => PositionType::Relative),
            vec![
                FlexMultiColorElement::new(
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 0),
                    style!(position_type => PositionType::Absolute, size => size!(auto; 20, px), position => rect!(d!(0), d!(0), d!(auto), d!(0))),
                    vec![],
                ),
                FlexMultiColorElement::new(
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 100),
                    style!(position_type => PositionType::Absolute, size => size!(auto; 20, px), position => rect!(d!(auto), d!(0), d!(0), d!(0))),
                    vec![],
                ),
            ],
        );

        Box::new(Self { flex })
    }
}

impl Element<CakeModel> for MainWindowMidi {
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
