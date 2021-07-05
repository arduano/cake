use imgui::Ui;
use stretch::{
    node::{Node, Stretch},
    result::Layout,
    style::Style,
    Error,
};

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
