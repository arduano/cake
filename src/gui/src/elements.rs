use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use imgui::{
    im_str, ChildWindow, ImColor32, ItemHoveredFlags, MouseButton, MouseCursor, Ui, WindowFlags,
};
use stretch::{
    node::{Node, Stretch},
    result::Layout,
    style::Style,
    Error,
};

use crate::{
    animation::{OneWayEase, VelocityEase},
    util::Lerp,
};

pub trait Element<Model> {
    fn set_layout(&mut self, node: Node);
    fn get_layout(&self) -> Option<Node>;
    fn last_layout<'a>(&self, stretch: &'a Stretch) -> &'a Layout {
        stretch
            .layout(self.get_layout().unwrap())
            .expect("Layout computation failed")
    }

    fn layout(
        &mut self,
        stretch: &mut Stretch,
        model: &Arc<Mutex<Box<Model>>>,
    ) -> Result<Node, Error> {
        let node = self.compute_layout(stretch, model)?;
        self.set_layout(node);

        Ok(node)
    }

    fn compute_layout(
        &mut self,
        stretch: &mut Stretch,
        model: &Arc<Mutex<Box<Model>>>,
    ) -> Result<Node, Error>;
    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &Arc<Mutex<Box<Model>>>);
}

pub trait BasicContainer<Model>: Element<Model> {
    fn children(&self) -> &Vec<Box<dyn Element<Model>>>;
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element<Model>>>;
    fn style(&self) -> Style;

    fn compute_layout_base(
        &mut self,
        stretch: &mut Stretch,
        model: &Arc<Mutex<Box<Model>>>,
    ) -> Result<Node, Error> {
        let mut children = Vec::new();
        for c in self.children_mut().iter_mut() {
            children.push(c.layout(stretch, model)?);
        }

        let node = stretch.new_node(self.style(), children);

        node
    }

    fn render_children(&mut self, stretch: &Stretch, ui: &Ui, model: &Arc<Mutex<Box<Model>>>) {
        for c in self.children_mut().iter_mut() {
            c.render(stretch, ui, model);
        }
    }
}

pub struct ShapeElement<Model> {
    children: Vec<Box<dyn Element<Model>>>,
    color: imgui::ImColor32,
    style: Style,
    last_layout: Option<Node>,
}

impl<Model> ShapeElement<Model> {
    pub fn new(
        color: imgui::ImColor32,
        style: Style,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(ShapeElement {
            children,
            color,
            style,
            last_layout: None,
        })
    }
}

impl<Model> Element<Model> for ShapeElement<Model> {
    fn compute_layout(
        &mut self,
        stretch: &mut Stretch,
        model: &Arc<Mutex<Box<Model>>>,
    ) -> Result<Node, Error> {
        let mut children = Vec::new();
        for c in self.children.iter_mut() {
            children.push(c.layout(stretch, model)?);
        }
        let node = stretch.new_node(self.style, children);

        node
    }

    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &Arc<Mutex<Box<Model>>>) {
        let layout = self.last_layout(stretch);
        ui.get_window_draw_list()
            .add_rect(
                [layout.location.x, layout.location.y],
                [
                    layout.location.x + layout.size.width,
                    layout.location.y + layout.size.height,
                ],
                self.color,
            )
            .filled(true)
            .build();

        for c in self.children.iter_mut() {
            c.render(stretch, ui, model);
        }
    }

    fn set_layout(&mut self, node: Node) {
        self.last_layout = Some(node);
    }

    fn get_layout(&self) -> Option<Node> {
        self.last_layout
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

pub struct RectangleRippleButton<Model, F: 'static + Fn()> {
    children: Vec<Box<dyn Element<Model>>>,
    background: imgui::ImColor32,
    style: Style,
    ripples: VecDeque<Ripple>,
    active: bool,
    last_layout: Option<Node>,

    on_click: F,
}

impl<Model, F: 'static + Fn()> RectangleRippleButton<Model, F> {
    pub fn new(
        background: imgui::ImColor32,
        style: Style,
        on_click: F,
        children: Vec<Box<dyn Element<Model>>>,
    ) -> Box<Self> {
        Box::new(RectangleRippleButton {
            children,
            background,
            style,
            ripples: VecDeque::new(),
            active: false,
            last_layout: None,
            on_click,
        })
    }
}

impl<Model, F: 'static + Fn()> Element<Model> for RectangleRippleButton<Model, F> {
    fn set_layout(&mut self, node: Node) {
        self.last_layout = Some(node);
    }

    fn get_layout(&self) -> Option<Node> {
        self.last_layout
    }

    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &Arc<Mutex<Box<Model>>>) {
        let layout = self.last_layout(stretch);

        self.render_children(stretch, ui, model);

        let p1 = [layout.location.x, layout.location.y];
        let p2 = [
            layout.location.x + layout.size.width,
            layout.location.y + layout.size.height,
        ];

        ui.set_cursor_pos(p1);

        let id = im_str!("Test Sized");

        ChildWindow::new(id).size(p2).build(ui, || {
            let dl = ui.get_window_draw_list();

            dl.add_rect(p1, p2, self.background).filled(true).build();

            for r in self.ripples.iter() {
                dl.add_circle(
                    [p1[0].lerp(&p2[0], r.x), p1[1].lerp(&p2[1], r.y)],
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
            (self.on_click)();
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

    fn compute_layout(
        &mut self,
        stretch: &mut Stretch,
        model: &Arc<Mutex<Box<Model>>>,
    ) -> Result<Node, Error> {
        self.compute_layout_base(stretch, model)
    }
}

impl<Model, F: 'static + Fn()> BasicContainer<Model> for RectangleRippleButton<Model, F> {
    fn children(&self) -> &Vec<Box<dyn Element<Model>>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element<Model>>> {
        &mut self.children
    }

    fn style(&self) -> Style {
        self.style
    }
}
