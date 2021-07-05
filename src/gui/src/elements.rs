use imgui::{im_str, ChildWindow, MouseButton, MouseCursor, Ui};
use stretch::{
    node::{Node, Stretch},
    result::Layout,
    style::Style,
    Error,
};

use crate::animation::VelocityEase;

pub trait Element<Model> {
    fn set_layout(&mut self, node: Node);
    fn get_layout(&self) -> Option<Node>;
    fn last_layout<'a>(&self, stretch: &'a Stretch) -> &'a Layout {
        stretch
            .layout(
                self.get_layout()
                    .expect("Layout wasn't comuted before render"),
            )
            .expect("Layout computation failed")
    }

    fn layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        let node = self.compute_layout(stretch, model)?;
        self.set_layout(node);

        Ok(node)
    }

    fn compute_layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error>;
    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &mut Model);
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
    fn compute_layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        let mut children = Vec::new();
        for c in self.children.iter_mut() {
            children.push(c.layout(stretch, model)?);
        }
        let node = stretch.new_node(self.style, children);

        node
    }

    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &mut Model) {
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

pub struct ClickableShapeElementTest<Model> {
    children: Vec<Box<dyn Element<Model>>>,
    color: imgui::ImColor32,
    last_layout: Option<Node>,
    active: bool,
    width: VelocityEase,
}

impl<Model> ClickableShapeElementTest<Model> {
    pub fn new(color: imgui::ImColor32, children: Vec<Box<dyn Element<Model>>>) -> Box<Self> {
        let mut width = VelocityEase::new(60.0);
        width.duration = 1.0;
        width.clamp_to_ends = false;

        Box::new(ClickableShapeElementTest {
            children,
            color,
            last_layout: None,
            active: false,
            width,
        })
    }
}

impl<Model> Element<Model> for ClickableShapeElementTest<Model> {
    fn compute_layout(&mut self, stretch: &mut Stretch, model: &mut Model) -> Result<Node, Error> {
        let mut children = Vec::new();
        for c in self.children.iter_mut() {
            children.push(c.layout(stretch, model)?);
        }

        {
            use stretch::geometry::Size;
            use stretch::style::*;
            let style = Style {
                size: Size {
                    width: Dimension::Points(self.width.value()),
                    height: Dimension::Points(50.0),
                },
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..Default::default()
            };

            let node = stretch.new_node(style, children);

            node
        }
    }

    fn render(&mut self, stretch: &Stretch, ui: &Ui, model: &mut Model) {
        let layout = self.last_layout(stretch);

        let p1 = [layout.location.x, layout.location.y];
        let p2 = [
            layout.location.x + layout.size.width,
            layout.location.y + layout.size.height,
        ];

        ui.set_cursor_pos(p1);

        ChildWindow::new(im_str!("Test Sized"))
            .size(p2)
            .build(ui, || {
                ui.get_window_draw_list()
                    .add_rect(p1, p2, self.color)
                    .filled(true)
                    .build();
            });

        if ui.is_item_hovered()
            || ui.is_item_active()
            || ui.is_item_focused()
            || ui.is_item_clicked(MouseButton::Left)
            || ui.is_mouse_hovering_rect(p1, p2)
        {
            ui.set_mouse_cursor(Some(MouseCursor::Hand));
        }

        if ui.is_item_clicked(MouseButton::Left) {
            if self.active {
                self.width.set_end(60.0);
            } else {
                self.width.set_end(300.0);
            }
            self.active = !self.active;
        }

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
