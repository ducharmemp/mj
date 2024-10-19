/// Create a new actor in an [`hashbrown::HashMap`]
///
/// The new actor is created and its [`ActorOwn`] reference is stored
/// in the provided [`hashbrown::HashMap`].  The termination notification
/// handler is set up to remove the reference from the map when the
/// actor terminates.  So this takes care of all the child actor
/// housekeeping for simple cases.
///
/// So assuming `self.children` is your [`hashbrown::HashMap`] instance, the
/// call will take one of these forms:
///
/// ```ignore
/// let actor = actor_in_map!(self.children, cx, 1, Type::init(args...));
/// let actor = actor_in_map!(self.children, cx, 1, <path::Type>::init(args...));
/// ```
///
/// If you need to monitor failures, then add a `Ret<StopCause>`
/// instance to the end of the macro arguments.  For example:
///
/// ```ignore
/// let actor = actor_in_map!(
///     self.children, cx, <path::Type>::init(args...),
///     ret_some_to!([cx], |this, cx, cause: StopCause| {
///         ...error handling...
///     }));
/// ```
///
/// Implemented using [`hashbrown::HashMap::insert`].
///
#[macro_export]
macro_rules! actor_in_map {
    ($self:ident.$children:ident, $cx:expr, $key:expr, $type:ident :: $init:ident($($x:expr),* $(,)? )) => {{
        $crate::actor_in_map!($self.$children, $cx, $key, <$type>::$init($($x),*), stakker::Ret::new(|_| {}))
    }};
    ($self:ident.$children:ident, $cx:expr, $key:expr, <$type:ty> :: $init:ident($($x:expr),* $(,)? )) => {{
        $crate::actor_in_map!($self.$children, $cx, $key, <$type>::$init($($x),*), stakker::Ret::new(|_| {}))
    }};
    ($self:ident.$children:ident, $cx:expr, $key:expr, $type:ident :: $init:ident($($x:expr),* $(,)? ), $notify:expr) => {{
        $crate::actor_in_map!($self.$children, $cx, $key, <$type>::$init($($x),*), $notify)
    }};
    ($self:ident.$children:ident, $cx:expr, $key:expr, <$type:ty> :: $init:ident($($x:expr),* $(,)? ), $notify:expr) => {{
        let notify = $notify;
        let key = $key;
        let parent = $cx.this().clone();
        let core = $cx.access_core();
        let actor = $self.$children.add(core, parent, key, |this| &mut this.$children, notify);
        stakker::call!([actor], <$type>::$init($($x),*));
        actor
    }};
}

/// Create a new actor in an [`hashbrown::HashMap`]. This does not initialize the actor, similar to
/// actor_new!
///
/// The new actor is created and its [`ActorOwn`] reference is stored
/// in the provided [`hashbrown::HashMap`].  The termination notification
/// handler is set up to remove the reference from the map when the
/// actor terminates.  So this takes care of all the child actor
/// housekeeping for simple cases.
///
/// So assuming `self.children` is your [`hashbrown::HashMap`] instance, the
/// call will take one of these forms:
///
/// ```ignore
/// let actor = actor_new_in_map!(self.children, cx, 1, Type));
/// let actor = actor_new_in_map!(self.children, cx, 1, <path::Type>);
/// ```
///
/// If you need to monitor failures, then add a `Ret<StopCause>`
/// instance to the end of the macro arguments.  For example:
///
/// ```ignore
/// let actor = actor_new_in_map!(
///     self.children, cx, <path::Type>::init(args...),
///     ret_some_to!([cx], |this, cx, cause: StopCause| {
///         ...error handling...
///     }));
/// ```
///
/// Implemented using [`hashbrown::HashMap::insert`].
///
#[macro_export]
macro_rules! actor_new_in_map {
    ($self:ident.$children:ident, $cx:expr, $key:expr) => {{
        $crate::actor_new_in_map!($self.$children, $cx, $key, stakker::Ret::new(|_| {}))
    }};
    ($self:ident.$children:ident, $cx:expr, $key:expr, $notify:expr) => {{
        let notify = $notify;
        let key = $key;
        let parent = $cx.this().clone();
        let core = $cx.access_core();
        let actor = $self
            .$children
            .add(core, parent, key, |this| &mut this.$children, notify);
        actor
    }};
}
