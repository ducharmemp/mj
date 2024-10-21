use stakker::{call, ret_some_do, Actor, CX};

use super::DomEntry;

impl DomEntry {
    pub(super) fn merge_with_sibling(&mut self, cx: CX![]) {
        let text_content = self.myself.text_contents();
        if text_content.is_none() {
            return;
        }
        let text_content = text_content.unwrap();
        let this_id = self.id;
        let parent = self.parent.clone();

        let sibling_cb = ret_some_do!(move |sibling: Option<Actor<DomEntry>>| {
            if let Some(sibling) = sibling {
                let append_to = sibling.clone();
                let sibling_text_node_cb = ret_some_do!(move |sibling_is_text: bool| {
                    if sibling_is_text {
                        call!([append_to], append_text_content(text_content));
                        // parent.map(|p| call!([p], inner_remove_child(this_id)));
                    }
                });
                call!([sibling], is_text(sibling_text_node_cb))
            }
        });
        call!([cx], previous_sibling(sibling_cb));
    }
}
