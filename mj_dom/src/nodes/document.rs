use ecow::EcoString;
use hashbrown::HashMap;
use mj_utilities::actor_own_map::ActorOwnMap;
use stakker::Actor;

use crate::parser::NodeId;

use super::dom_entry::DomEntry;

pub struct MjDocument {
    inner_id_store: ActorOwnMap<NodeId, DomEntry>,
    id_cache: HashMap<EcoString, Vec<Actor<DomEntry>>>,
    scripts: Vec<Actor<DomEntry>>,
}

impl MjDocument {}
