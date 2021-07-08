use std::{collections::VecDeque, ops::Deref};

use imgui::{ChildWindow, Id, ImColor32, ItemHoveredFlags, MouseButton, MouseCursor, Ui};
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
    pub children: Vec<Box<dyn Element<Model>>>,
    pub style: Style,
    pub last_layout: Option<Node>,
}

impl<Model> FlexElement<Model> {
    pub fn new(style: Style, children: Vec<Box<dyn Element<Model>>>) -> Box<Self> {
        Box::new(FlexElement {
            children,
            style,
            last_layout: None,
        })
    }

    pub fn last_layout<'a>(&self, stretch: &'a Stretch) -> &'a Layout {
        stretch
            .layout(self.last_layout.unwrap())
            .expect("Layout computation failed")
    }

    pub fn render_children(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
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

pub struct FGetCol<Model>(Box<dyn Fn(&mut Model) -> imgui::ImColor32>);
impl<Model> FGetCol<Model> {
    pub fn new<T: 'static + Fn(&mut Model) -> imgui::ImColor32>(f: T) -> Self {
        Self(Box::new(f))
    }
}
impl<Model> Deref for FGetCol<Model> {
    type Target = Box<dyn Fn(&mut Model) -> imgui::ImColor32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Model> Into<FGetCol<Model>> for imgui::ImColor32 {
    fn into(self) -> FGetCol<Model> {
        FGetCol(Box::new(move |_| self))
    }
}

pub struct FlexColorElement<Model> {
    flex: FlexElement<Model>,
    color: FGetCol<Model>,
}

impl<Model> FlexColorElement<Model> {
    pub fn new<FCol: Into<FGetCol<Model>>>(
        color: FCol,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(FlexColorElement {
            flex: FlexElement {
                children,
                style,
                last_layout: None,
            },
            color: color.into(),
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
            .add_rect(p1, p2, (self.color)(model))
            .filled(true)
            .build();

        self.flex.render_children(p1, stretch, ui, model);
    }
}

pub struct FlexMultiColorElement<Model> {
    flex: FlexElement<Model>,
    color_tl: FGetCol<Model>,
    color_tr: FGetCol<Model>,
    color_br: FGetCol<Model>,
    color_bl: FGetCol<Model>,
}

impl<Model> FlexMultiColorElement<Model> {
    pub fn new<FCol: Into<FGetCol<Model>>>(
        color_tl: FCol,
        color_tr: FCol,
        color_br: FCol,
        color_bl: FCol,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(FlexMultiColorElement {
            flex: FlexElement {
                children,
                style,
                last_layout: None,
            },
            color_tl: color_tl.into(),
            color_tr: color_tr.into(),
            color_br: color_br.into(),
            color_bl: color_bl.into(),
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

impl<Model> Element<Model> for FlexMultiColorElement<Model> {
    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        self.flex.layout(stretch, model)
    }

    fn render(&mut self, anchor: [f32; 2], stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let [p1, p2, _] = self.flex.get_layout_points(anchor, stretch);
        ui.get_window_draw_list().add_rect_filled_multicolor(
            p1,
            p2,
            (self.color_tl)(model),
            (self.color_tr)(model),
            (self.color_br)(model),
            (self.color_bl)(model),
        );

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
    id: Id<'static>,

    on_click: F,
}

impl<Model, F: 'static + Fn(&mut Model)> RippleButton<Model, F> {
    pub fn new<FCol: Into<FGetCol<Model>>>(
        background: FCol,
        style: Style,
        on_click: F,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(RippleButton {
            flex: FlexColorElement {
                color: background.into(),
                flex: {
                    FlexElement {
                        children,
                        style,
                        last_layout: None,
                    }
                },
            },
            id: rand_im_id(),
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

        ChildWindow::new(self.id).size(size).build(ui, || {
            let dl = ui.get_window_draw_list();

            dl.add_rect(p1, p2, (self.flex.color)(model))
                .filled(true)
                .build();

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
    selected_col: FGetCol<Model>,
    background: FGetCol<Model>,
}

impl<Model, FClick: 'static + Fn(&mut Model), FSelected: 'static + Fn(&mut Model) -> bool>
    ToggleButton<Model, FClick, FSelected>
{
    pub fn new<FCol1: Into<FGetCol<Model>>, FCol2: Into<FGetCol<Model>>>(
        background: FCol1,
        selected_col: FCol2,
        style: Style,
        get_selected: FSelected,
        on_click: FClick,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(ToggleButton {
            ripple_button: RippleButton::new(ImColor32::TRANSPARENT, style, on_click, children),
            get_selected,
            selected_col: selected_col.into(),
            background: background.into(),
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
            dl.add_rect(p1, p2, (self.background)(model))
                .filled(true)
                .build();
            if (self.get_selected)(model) {
                dl.add_rect(p1, p2, (self.selected_col)(model))
                    .filled(true)
                    .build();
            }
        }

        self.ripple_button.render(anchor, stretch, ui, model);
    }
}

pub struct FGetTex<Model>(Box<dyn Fn(&mut Model) -> imgui::TextureId>);
impl<Model> FGetTex<Model> {
    pub fn new<T: 'static + Fn(&mut Model) -> imgui::TextureId>(f: T) -> Self {
        Self(Box::new(f))
    }
}
impl<Model> Deref for FGetTex<Model> {
    type Target = Box<dyn Fn(&mut Model) -> imgui::TextureId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Model> Into<FGetTex<Model>> for imgui::TextureId {
    fn into(self) -> FGetTex<Model> {
        FGetTex(Box::new(move |_| self))
    }
}

pub struct FlexImageElement<Model> {
    flex: FlexElement<Model>,
    texture: FGetTex<Model>,
}

impl<Model> FlexImageElement<Model> {
    pub fn new<FTex: Into<FGetTex<Model>>>(
        texture: FTex,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(FlexImageElement {
            flex: FlexElement {
                children,
                style,
                last_layout: None,
            },
            texture: texture.into(),
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
        imgui::Image::new((self.texture)(model), size).build(&ui);

        self.flex.render_children(p1, stretch, ui, model);
    }
}
