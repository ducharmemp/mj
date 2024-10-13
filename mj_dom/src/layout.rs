use taffy::{prelude::length, Display, NodeId, Size, Style, TaffyTree};
use vello::{
    kurbo::{Affine, RoundedRect, Stroke},
    peniko::Color,
    Scene,
};

pub struct LayoutTree {
    root: NodeId,
    inner: TaffyTree,
}

impl LayoutTree {
    pub fn set_content_area(&mut self, width: f32, height: f32) {
        self.inner
            .compute_layout(
                self.root,
                Size {
                    width: length(width),
                    height: length(height),
                },
            )
            .unwrap()
    }

    pub fn draw(self, scene: &mut Scene) {
        fn walk(scene: &mut Scene, taffy: &TaffyTree, node: taffy::NodeId) {
            let layout = taffy.layout(node).unwrap();
            let stroke = Stroke::new(6.0);
            let rect = RoundedRect::new(
                layout.location.x.into(),
                layout.location.y.into(),
                layout.size.width.into(),
                layout.size.height.into(),
                0.0,
            );
            let rect_stroke_color = Color::rgb(0.9804, 0.702, 0.5294);
            scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);
            for child in taffy.children(node).unwrap() {
                walk(scene, taffy, child);
            }
        }

        walk(scene, &self.inner, self.root);
    }
}

impl From<&DomTree> for LayoutTree {
    fn from(value: &DomTree) -> Self {
        let mut taffy = TaffyTree::new();
        let container_id = taffy.new_leaf(Default::default()).unwrap();
        let body = value.body();
        if body.is_none() {
            dbg!("No body");
            return Self {
                root: container_id,
                inner: taffy,
            };
        }
        let body = body.unwrap();
        fn descent(dom: &DomTree, node: &DomEntry, builder: &mut TaffyTree) -> taffy::NodeId {
            let new_leaf = builder
                .new_leaf(Style {
                    size: Size {
                        width: length(100.0),
                        height: length(100.0),
                    },
                    display: Display::Block,
                    ..Default::default()
                })
                .expect("could not construct leaf");
            for child in dom.iter_children(node) {
                let child_id = descent(dom, child, builder);
                builder.add_child(new_leaf, child_id);
            }
            new_leaf
        }

        let container_id = descent(&value, body, &mut taffy);
        Self {
            root: container_id,
            inner: taffy,
        }
    }
}
