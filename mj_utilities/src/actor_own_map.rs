use std::hash::Hash;

use hashbrown::HashMap;
use stakker::{ret, ret_some_do, Actor, ActorOwn, Core, Ret, StopCause};

pub struct ActorOwnMap<Key: Eq + Hash + Clone + 'static, ActorType: 'static>(
    HashMap<Key, ActorOwn<ActorType>>,
);

impl<Key: Eq + Hash + Clone + 'static, ActorType: 'static> ActorOwnMap<Key, ActorType> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add<P>(
        &mut self,
        core: &mut Core,
        parent: Actor<P>,
        key: Key,
        get_map: impl for<'a> FnOnce(&'a mut P) -> &'a mut Self + 'static,
        notify: Ret<StopCause>,
    ) -> Actor<ActorType> {
        let refkey = key.clone();
        let parid = parent.id();
        let actorown = ActorOwn::new(
            core,
            ret_some_do!(move |cause| {
                let parent2 = parent.clone();
                parent.defer(move |s| {
                    parent2.apply(s, move |this, _| {
                        get_map(this).0.remove(&refkey);
                    });
                });
                ret!([notify], cause);
            }),
            parid,
        );
        let actor = actorown.clone();
        self.0.insert(key, actorown);
        actor
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut ActorOwn<ActorType>> {
        self.0.get_mut(key)
    }
    pub fn get(&mut self, key: &Key) -> Option<&ActorOwn<ActorType>> {
        self.0.get(key)
    }
    pub fn get_many_mut<const N: usize>(
        &mut self,
        ks: [&Key; N],
    ) -> [Option<&mut ActorOwn<ActorType>>; N] {
        self.0.get_many_mut(ks)
    }
}
