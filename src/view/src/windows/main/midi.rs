use std::sync::{Arc, Mutex};

use gui::elements::{Element, FlexElement};
use imgui::Ui;
use stretch::{node::Node, style::Style, Error, Stretch};

use crate::{model::CakeModel, palette};

pub struct MainWindowMidi {
    flex: Box<dyn Element<CakeModel>>,
}

pub struct RenderElement {
    flex: Box<FlexElement<CakeModel>>,
}

impl RenderElement {
    pub fn new(style: Style) -> Box<Self> {
        Box::new(RenderElement {
            flex: FlexElement::new(style, vec![]),
        })
    }

    pub fn render_children(
        &mut self,
        anchor: [f32; 2],
        stretch: &Stretch,
        ui: &Ui,
        model: &mut CakeModel,
    ) {
        self.flex.render_children(anchor, stretch, ui, model)
    }

    pub fn get_layout_points(&self, anchor: [f32; 2], stretch: &Stretch) -> [[f32; 2]; 3] {
        self.flex.get_layout_points(anchor, stretch)
    }
}

impl Element<CakeModel> for RenderElement {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut CakeModel) -> Result<Node, Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut CakeModel) {
        let [p1, _, size] = self.flex.get_layout_points(anchor, stretch);

        ui.set_cursor_pos(p1);
        imgui::Image::new(model.view.renderer.texture_id, size).build(&ui);

        model.view.renderer.last_size = size;
    }
}

impl MainWindowMidi {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Box<Self> {
        use gui::{
            d,
            elements::{FlexColorElement, FlexElement, FlexImageElement, FlexMultiColorElement},
            rect, rgba, rgbaf, size, style,
        };
        use stretch::style::{AlignItems, FlexDirection, PositionType};

        let texid = { model.lock().unwrap().view.renderer.texture_id };

        let flex = FlexColorElement::new(
            rgba!(0, 0, 0, 0),
            style!(size => size!(100, %; 100, %), flex_basis => d!(100, %), position_type => PositionType::Relative),
            vec![
                RenderElement::new(
                    style!(position_type => PositionType::Absolute, position => rect!(d!(0), d!(0), d!(0), d!(0))),
                ),
                FlexMultiColorElement::new(
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 0),
                    style!(position_type => PositionType::Absolute, size => size!(auto; 10, px), position => rect!(d!(0), d!(0), d!(auto), d!(0))),
                    vec![],
                ),
                FlexMultiColorElement::new(
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 0),
                    rgba!(0, 0, 0, 100),
                    rgba!(0, 0, 0, 100),
                    style!(position_type => PositionType::Absolute, size => size!(auto; 10, px), position => rect!(d!(auto), d!(0), d!(0), d!(0))),
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
