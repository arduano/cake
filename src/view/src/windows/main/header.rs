use std::sync::{Arc, Mutex};

use gui::elements::Element;

use crate::{model::CakeModel, palette};

pub struct MainWindowHeader {
    flex: Box<dyn Element<CakeModel>>,
}

impl MainWindowHeader {
    pub fn new(model: &Arc<Mutex<CakeModel>>) -> Box<Self> {
        use color::{Deg, Hsv, ToRgb};
        use gui::{
            d,
            elements::{
                FlexColorElement, FlexElement, FlexImageElement, RippleButton, ToggleButton,
            },
            rect, rgb, rgba, rgbaf, size, style,
            util::ToImColor,
        };
        use stretch::style::{AlignItems, FlexDirection, JustifyContent};

        let model = model.lock().unwrap();
        let textures = &model.view.textures;
        let flex = FlexColorElement::new(
            palette!(bg_light),
            style!(size => size!(100, %; auto), flex_shrink => 0.0, flex_direction => FlexDirection::Column, align_items => AlignItems::Stretch),
            vec![
                // Row 1
                FlexColorElement::new(
                    rgba!(0, 0, 0, 0),
                    style!(size => size!(100, %; 40, px)),
                    vec![
                        FlexColorElement::new(
                            palette!(primary),
                            style!(),
                            vec![
                                ToggleButton::new(
                                    rgba!(0, 0, 0, 0),
                                    rgbaf!(1, 1, 1, 0.2),
                                    style!(size => size!(40, px; 40, px), padding => rect!(d!(5, px))),
                                    move |model: &mut CakeModel| -> bool { model.view.paused },
                                    move |model: &mut CakeModel| {
                                        model.view.paused = true;
                                    },
                                    vec![FlexImageElement::new(
                                        textures.pause_button,
                                        style!(size => size!(100, %; 100, %)),
                                        vec![],
                                    )],
                                ),
                                ToggleButton::new(
                                    rgba!(0, 0, 0, 0),
                                    rgbaf!(1, 1, 1, 0.2),
                                    style!(size => size!(40, px; 40, px), padding => rect!(d!(5, px))),
                                    move |model: &mut CakeModel| -> bool { !model.view.paused },
                                    move |model: &mut CakeModel| {
                                        model.view.paused = false;
                                    },
                                    vec![FlexImageElement::new(
                                        textures.play_button,
                                        style!(size => size!(100, %; 100, %)),
                                        vec![],
                                    )],
                                ),
                            ],
                        ),
                        FlexColorElement::new(
                            rgba!(0, 0, 0, 0),
                            style!(flex_basis => d!(100, %), padding => rect!(d!(15, px))),
                            vec![FlexColorElement::new(
                                palette!(primary),
                                style!(size => size!(100, %; 100, %)),
                                vec![],
                            )],
                        ),
                    ],
                ),
                // Row 2
                FlexColorElement::new(
                    rgba!(0, 0, 0, 0),
                    style!(size => size!(100, %; 40, px)),
                    vec![],
                ),
            ],
        );

        Box::new(Self { flex })
    }
}

impl Element<CakeModel> for MainWindowHeader {
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
