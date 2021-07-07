use std::collections::VecDeque;

use imgui::{ChildWindow, ImColor32, ItemHoveredFlags, MouseButton, MouseCursor, Ui};
use stretch::{
    node::{Node, Stretch},
    result::Layout,
    style::Style,
    Error,
};

use crate::{
    animation::OneWayEase,
    util::{rand_im_id, Lerp},
};

pub trait Element<Model> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error>;

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model);
}

pub struct FlexElement<Model> {
    children: Vec<Box<dyn Element<Model>>>,
    style: Style,
    last_layout: Option<Node>,
}

impl<Model> FlexElement<Model> {
    pub fn new(style: Style, children: Vec<Box<dyn Element<Model>>>) -> Box<Self> {
        Box::new(FlexElement {
            children,
            style,
            last_layout: None,
        })
    }

    fn last_layout<'a>(&self, stretch: &'a Stretch) -> &'a Layout {
        stretch
            .layout(self.last_layout.unwrap())
            .expect("Layout computation failed")
    }

    fn render_children(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        for c in self.children.iter_mut() {
            c.render(anchor, stretch, ui, model);
        }
    }

    pub fn get_layout_points(&self, anchor: [f32; 2], stretch: &Stretch) -> [[f32; 2]; 3] {
        let layout = self.last_layout(stretch);

        let p1 = [layout.location.x + anchor[0], layout.location.y + anchor[1]];
        let p2 = [p1[0] + layout.size.width, p1[1] + layout.size.height];
        let size = [layout.size.width, layout.size.height];

        [p1, p2, size]
    }
}

impl<Model> Element<Model> for FlexElement<Model> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        let mut children = Vec::new();
        for c in self.children.iter_mut() {
            children.push(c.layout(stretch, model)?);
        }
        let node = stretch.new_node(self.style, children)?;

        self.last_layout = Some(node);

        Ok(node)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        self.render_children(anchor, stretch, ui, model);
    }
}

pub struct FlexColorElement<Model> {
    flex: FlexElement<Model>,
    color: imgui::ImColor32,
}

impl<Model> FlexColorElement<Model> {
    pub fn new(
        color: imgui::ImColor32,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(FlexColorElement {
            flex: FlexElement {
                children,
                style,
                last_layout: None,
            },
            color,
        })
    }

    pub fn render_children(
        &mut self,
        anchor: [f32; 2],
        stretch: &Stretch,
        ui: &Ui,
        model: &mut Model,
    ) {
        self.flex.render_children(anchor, stretch, ui, model)
    }

    pub fn get_layout_points(&self, anchor: [f32; 2], stretch: &Stretch) -> [[f32; 2]; 3] {
        self.flex.get_layout_points(anchor, stretch)
    }
}

impl<Model> Element<Model> for FlexColorElement<Model> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let [p1, p2, _] = self.flex.get_layout_points(anchor, stretch);
        ui.get_window_draw_list()
            .add_rect(p1, p2, self.color)
            .filled(true)
            .build();

        self.flex.render_children(p1, stretch, ui, model);
    }
}

struct Ripple {
    x: f32,
    y: f32,
    rad: OneWayEase<f32>,
    col: OneWayEase<ImColor32>,
}

impl Ripple {
    pub fn new(x: f32, y: f32) -> Self {
        Ripple {
            x,
            y,
            col: OneWayEase::new(
                ImColor32::from_rgba(255, 255, 255, (255.0 * 0.4) as u8),
                ImColor32::from_rgba(0, 0, 0, 0),
                0.3,
                0.1,
            ),
            rad: OneWayEase::new_started(0.0, 2.0, 0.4, 0.0),
        }
    }
}

pub struct RippleButton<Model, F: 'static + Fn(&mut Model)> {
    flex: FlexColorElement<Model>,
    ripples: VecDeque<Ripple>,
    active: bool,

    on_click: F,
}

impl<Model, F: 'static + Fn(&mut Model)> RippleButton<Model, F> {
    pub fn new(
        background: imgui::ImColor32,
        style: Style,
        on_click: F,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(RippleButton {
            flex: FlexColorElement {
                color: background,
                flex: {
                    FlexElement {
                        children,
                        style,
                        last_layout: None,
                    }
                },
            },
            ripples: VecDeque::new(),
            active: false,
            on_click,
        })
    }
}

impl<Model, F: 'static + Fn(&mut Model)> Element<Model> for RippleButton<Model, F> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let [p1, p2, size] = self.flex.get_layout_points(anchor, stretch);

        self.flex.render_children(p1, stretch, ui, model);

        ui.set_cursor_pos(p1);

        ChildWindow::new(rand_im_id()).size(size).build(ui, || {
            let dl = ui.get_window_draw_list();

            dl.add_rect(p1, p2, self.flex.color).filled(true).build();

            for r in self.ripples.iter() {
                dl.add_circle(
                    [p1[0].lerpv(p2[0], r.x), p1[1].lerpv(p2[1], r.y)],
                    r.rad.value() * (p2[0] - p1[0]) * 2.0,
                    r.col.value(),
                )
                .filled(true)
                .build();
            }
        });

        if ui.is_item_clicked(MouseButton::Left) {
            let pos = ui.io().mouse_pos;
            let x = (pos[0] - p1[0]) / (p2[0] - p1[0]);
            let y = (pos[1] - p1[1]) / (p2[1] - p1[1]);
            self.ripples.push_front(Ripple::new(x, y));
            self.active = true;
            (self.on_click)(model);
        }

        if ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM) {
            ui.set_mouse_cursor(Some(MouseCursor::Hand));
        }

        if !ui.is_mouse_down(MouseButton::Left) {
            if self.active {
                match self.ripples.front_mut() {
                    Some(r) => {
                        r.col.start();
                    }
                    None => {}
                }
                self.active = false;
            }
        }

        loop {
            match self.ripples.back() {
                None => break,
                Some(ripple) => {
                    if ripple.col.ended() {
                        self.ripples.pop_back();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

pub struct ToggleButton<
    Model,
    FClick: 'static + Fn(&mut Model),
    FSelected: 'static + Fn(&mut Model) -> bool,
> {
    ripple_button: Box<RippleButton<Model, FClick>>,
    get_selected: FSelected,
    selected_col: ImColor32,
    background: ImColor32,
}

impl<Model, FClick: 'static + Fn(&mut Model), FSelected: 'static + Fn(&mut Model) -> bool>
    ToggleButton<Model, FClick, FSelected>
{
    pub fn new(
        background: ImColor32,
        selected_col: ImColor32,
        style: Style,
        get_selected: FSelected,
        on_click: FClick,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(ToggleButton {
            ripple_button: RippleButton::new(ImColor32::TRANSPARENT, style, on_click, children),
            get_selected,
            selected_col,
            background,
        })
    }
}

impl<Model, FClick: 'static + Fn(&mut Model), FSelected: 'static + Fn(&mut Model) -> bool>
    Element<Model> for ToggleButton<Model, FClick, FSelected>
{
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        self.ripple_button.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let [p1, p2, _] = self.ripple_button.flex.get_layout_points(anchor, stretch);

        {
            let dl = ui.get_window_draw_list();
            dl.add_rect(p1, p2, self.background).filled(true).build();
            if (self.get_selected)(model) {
                dl.add_rect(p1, p2, self.selected_col).filled(true).build();
            }
        }

        self.ripple_button.render(anchor, stretch, ui, model);
    }
}

pub struct FlexImageElement<Model> {
    flex: FlexElement<Model>,
    texture: imgui::TextureId,
}

impl<Model> FlexImageElement<Model> {
    pub fn new(
        texture: imgui::TextureId,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(FlexImageElement {
            flex: FlexElement {
                children,
                style,
                last_layout: None,
            },
            texture,
        })
    }

    pub fn render_children(
        &mut self,
        anchor: [f32; 2],
        stretch: &Stretch,
        ui: &Ui,
        model: &mut Model,
    ) {
        self.flex.render_children(anchor, stretch, ui, model)
    }

    pub fn get_layout_points(&self, anchor: [f32; 2], stretch: &Stretch) -> [[f32; 2]; 3] {
        self.flex.get_layout_points(anchor, stretch)
    }
}

impl<Model> Element<Model> for FlexImageElement<Model> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let [p1, _, size] = self.flex.get_layout_points(anchor, stretch);

        ui.set_cursor_pos(p1);
        imgui::Image::new(self.texture, size).build(&ui);

        self.flex.render_children(p1, stretch, ui, model);
    }
}
