use std::collections::VecDeque;

use stakker::{call, fwd, ret, ret_do, ret_to, stop, Actor, Fwd, Ret, CX};

use crate::nodes::DomEntry;

#[derive(Clone, Copy)]
enum DomIteratorNextMovement {
    Right,
    Down,
}

pub struct ForwardDomIterator {
    stack: VecDeque<(usize, Actor<DomEntry>)>,
    max_visited: usize,
    current: (usize, Actor<DomEntry>),
    next_movement: DomIteratorNextMovement,
}

impl ForwardDomIterator {
    pub fn init(cx: CX![], start_node: Actor<DomEntry>) -> Option<Self> {
        Some(Self {
            stack: VecDeque::new(),
            max_visited: 0,
            current: (0, start_node),
            next_movement: DomIteratorNextMovement::Down,
        })
    }

    pub fn next(&mut self, cx: CX![], callback: Ret<Actor<DomEntry>>) {
        let (_, current) = &self.current;
        match self.next_movement {
            DomIteratorNextMovement::Down => {
                let ret = ret_to!([cx], recv_node(callback) as (Option<Actor<DomEntry>>));
                call!([current], first_child(ret));
            }
            DomIteratorNextMovement::Right => {
                let ret = ret_to!([cx], recv_node(callback) as (Option<Actor<DomEntry>>));
                call!([current], next_sibling(ret));
            }
        }
    }

    fn recv_node(
        &mut self,
        cx: CX![],
        callback: Ret<Actor<DomEntry>>,
        node: Option<Option<Actor<DomEntry>>>,
    ) {
        let node = node.flatten();
        let (current_id, current) = &self.current;

        match (node, self.next_movement) {
            (None, DomIteratorNextMovement::Down) => {
                self.next_movement = DomIteratorNextMovement::Right;
                self.next(cx, callback);
            }
            (None, DomIteratorNextMovement::Right) => {
                if *current_id >= self.max_visited {
                    ret!([callback], current.clone());
                }
                match self.stack.pop_back() {
                    Some(new_current) => {
                        self.current = new_current;
                    }
                    None => stop!(cx),
                }
            }
            (Some(entry), _) => {
                if *current_id >= self.max_visited {
                    ret!([callback], current.clone());
                }
                self.stack.push_back(self.current.clone());
                self.max_visited += 1;
                self.current = (self.max_visited, entry);
                self.next_movement = DomIteratorNextMovement::Down;
            }
        }
    }
}
