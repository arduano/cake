use std::sync::{Arc, Mutex};

use gui::elements::Element;

use crate::{model::CakeModel, palette};

pub struct MainWindowKeyboard {
    flex: Box<dyn Element<CakeModel>>,
}

impl MainWindowKeyboard {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Box<Self> {
        use gui::{
            d,
            elements::{FlexColorElement, FlexElement},
            rgb, rgbaf, size, style,
        };
        use stretch::style::{AlignItems, FlexDirection};

        let model = model.clone();

        let flex = FlexColorElement::new(
            palette!(bg_light),
            style!(size => size!(100, %; 150, px), flex_shrink => 0.0),
            vec![],
        );

        Box::new(Self { flex })
    }
}

impl Element<CakeModel> for MainWindowKeyboard {
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
