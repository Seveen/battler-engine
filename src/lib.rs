pub extern crate paste;

#[macro_export]
macro_rules! register_components {
    (
        index $index_type:ty,
        components { $( $component_type:ty ),* }
        spatial { $( $spatial_type:ty ),* }
    ) => {
        $crate::paste::paste! {
            use std::collections::{VecDeque, HashMap, HashSet};
            use std::fmt::Debug;
            use rstar::{AABB, RTree, RTreeObject, Envelope, PointDistance};

            #[cfg(feature = "serde_support")]
            use serde::{Serialize, Deserialize};

            #[derive(Debug)]
            #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
            pub struct GameState {
                $(
                    pub [<$component_type:lower>]: HashMap<$index_type, $component_type>,
                )*
                $(
                    pub [<$spatial_type:lower>]: HashMap<$index_type, $spatial_type>,
                )*
            }

            impl GameState {
                pub fn new() -> Self {
                    GameState {
                        $(
                            [<$component_type:lower>]: HashMap::new(),
                        )*
                        $(
                            [<$spatial_type:lower>]: HashMap::new(),
                        )*
                    }
                }

                pub fn clear(&mut self) {
                    $(
                        self.[<$component_type:lower>].clear();
                    )*
                    $(
                        self.[<$spatial_type:lower>].clear();
                    )*
                }

                $(
                    pub fn [<get_ $component_type:lower>](&self, id: $index_type) -> Option<&$component_type> {
                        self.[<$component_type:lower>].get(&id)
                    }

                )*
                $(
                    pub fn [<get_ $spatial_type:lower>](&self, id: $index_type) -> Option<&$spatial_type> {
                        self.[<$spatial_type:lower>].get(&id)
                    }
                )*

                pub fn commit_action(&mut self, action: &mut Action) {
                    $(
                        for (id, value) in action.updates.[<$component_type:lower>].drain() {
                            self.[<$component_type:lower>].insert(id, value);
                        }
                        for id in action.removals.[<$component_type:lower>].drain() {
                            self.[<$component_type:lower>].remove(&id);
                        }
                    )*
                    $(
                        for (id, value) in action.updates.[<$spatial_type:lower>].drain() {
                            self.[<$spatial_type:lower>].insert(id, value);
                        }
                        for id in action.removals.[<$spatial_type:lower>].drain() {
                            self.[<$spatial_type:lower>].remove(&id);
                        }
                    )*
                }

                pub fn into_action(mut self) -> Action {
                    let mut action = Action::new();
                    $(
                        for (id, value) in self.[<$component_type:lower>].drain() {
                            action.[<insert_ $component_type:lower>](id, value);
                        }
                    )*
                    $(
                        for (id, value) in self.[<$spatial_type:lower>].drain() {
                            action.[<insert_ $spatial_type:lower>](id, value);
                        }
                    )*
                    action
                }
            }

            #[derive(Debug)]
            struct RemovedComponents {
                $(
                    [<$component_type:lower>]: HashSet<$index_type>,
                )*
                $(
                    [<$spatial_type:lower>]: HashSet<$index_type>,
                )*
            }

            impl RemovedComponents {
                fn new() -> Self {
                    RemovedComponents {
                    $(
                        [<$component_type:lower>]: HashSet::new(),
                    )*
                    $(
                        [<$spatial_type:lower>]: HashSet::new(),
                    )*
                    }
                }

                fn clear(&mut self) {
                    $(
                        self.[<$component_type:lower>].clear();
                    )*
                    $(
                        self.[<$spatial_type:lower>].clear();
                    )*
                }
            }

            #[derive(Debug)]
            pub struct Action {
                updates: GameState,
                removals: RemovedComponents,
            }

            impl Action {
                pub fn new() -> Self {
                    Action {
                        updates: GameState::new(),
                        removals: RemovedComponents::new(),
                    }
                }

                pub fn clear(&mut self) {
                    self.updates.clear();
                    self.removals.clear();
                }

                $(
                    pub fn [<insert_ $component_type:lower>](&mut self, id: $index_type, value: $component_type) {
                        self.updates.[<$component_type:lower>].insert(id, value);
                    }

                    pub fn [<get_updated_ $component_type:lower>](&self) -> &HashMap<$index_type, $component_type> {
                        &self.updates.[<$component_type:lower>]
                    }

                    pub fn [<remove_ $component_type:lower>](&mut self, id: $index_type) {
                        self.removals.[<$component_type:lower>].insert(id);
                    }

                    pub fn [<get_removed_ $component_type:lower>](&self) -> &HashSet<$index_type> {
                        &self.removals.[<$component_type:lower>]
                    }
                )*
                $(
                    pub fn [<insert_ $spatial_type:lower>](&mut self, id: $index_type, value: $spatial_type) {
                        self.updates.[<$spatial_type:lower>].insert(id, value);
                    }

                    pub fn [<get_updated_ $spatial_type:lower>](&self) -> &HashMap<$index_type, $spatial_type> {
                        &self.updates.[<$spatial_type:lower>]
                    }

                    pub fn [<remove_ $spatial_type:lower>](&mut self, id: $index_type) {
                        self.removals.[<$spatial_type:lower>].insert(id);
                    }

                    pub fn [<get_removed_ $spatial_type:lower>](&self) -> &HashSet<$index_type> {
                        &self.removals.[<$spatial_type:lower>]
                    }
                )*

                pub fn remove_all(&mut self, id: EntityId) {
                    $(
                        self.[<remove_ $component_type:lower>](id);
                    )*
                    $(
                        self.[<remove_ $spatial_type:lower>](id);
                    )*
                }
            }

            pub struct FutureState<'a> {
                pub state: &'a GameState,
                pub action: &'a Action,
            }

            impl<'a> FutureState<'a> {
                $(
                    pub fn [<get_ $component_type:lower>](&self, id: $index_type) -> Option<&$component_type> {
                        if let Some(value) = self.action.updates.[<get_ $component_type:lower>](id) {
                            return Some(value);
                        }
                        if self.action.removals.[<$component_type:lower>].contains(&id) {
                            return None;
                        }
                        self.state.[<get_ $component_type:lower>](id)
                    }
                )*
                $(
                    pub fn [<get_ $spatial_type:lower>](&self, id: $index_type) -> Option<&$spatial_type> {
                        if let Some(value) = self.action.updates.[<get_ $spatial_type:lower>](id) {
                            return Some(value);
                        }
                        if self.action.removals.[<$spatial_type:lower>].contains(&id) {
                            return None;
                        }
                        self.state.[<get_ $spatial_type:lower>](id)
                    }
                )*
            }

            #[derive(Debug, PartialEq, Eq)]
            pub enum ActionStatus {
                Accept,
                Reject,
            }

            #[derive(Debug, PartialEq, Eq)]
            pub enum RuleStatus {
                KeepChecking,
                StopChecking,
            }

            pub type RuleFn<T> = fn(
                &Action,
                &GameState,
                $(
                    &RTree<[<$spatial_type TreeObject>]>,
                )*
            ) -> (ActionStatus, RuleStatus, Vec<T>);

            pub type ActionCreationFn<T> = fn(
                T,
                &GameState,
                &mut Action,
                $(
                    &RTree<[<$spatial_type TreeObject>]>,
                )*
            );

            pub type HookFn<E> = fn(
                &mut VecDeque<E>,
                &Action,
                &GameState,
                $(
                    &RTree<[<$spatial_type TreeObject>]>,
                )*
            );

            pub type HookWithouActionFn<E> = fn(
                &mut VecDeque<E>,
                &GameState,
                $(
                    &RTree<[<$spatial_type TreeObject>]>,
                )*
            );

            pub struct GameWorld<T, E> {
                populate_action: ActionCreationFn<T>,
                pub action: Action,
                pub state: GameState,
                rules: Vec<RuleFn<T>>,
                pending_actions: VecDeque<T>,
                follow_on_current: VecDeque<T>,
                follow_on_accepted: VecDeque<T>,
                follow_on_rejected: VecDeque<T>,
                hooks_on_accepted: Vec<HookFn<E>>,
                hooks_on_rejected: Vec<HookFn<E>>,
                hooks_after_commit: Vec<HookWithouActionFn<E>>,
                pub events_queue: VecDeque<E>,
                $(
                    pub [<spatial_ $spatial_type:lower>]: RTree<[<$spatial_type TreeObject>]>,
                )*
            }

            impl<T: Debug, E> GameWorld<T, E> {
                pub fn new(
                    rules: Vec<RuleFn<T>>,
                    populate_action: ActionCreationFn<T>,
                    hooks_on_accepted: Vec<HookFn<E>>,
                    hooks_on_rejected: Vec<HookFn<E>>,
                    hooks_after_commit: Vec<HookWithouActionFn<E>>,
                ) -> Self {
                    GameWorld::new_with_initial_state(rules, populate_action, hooks_on_accepted, hooks_on_rejected, hooks_after_commit, GameState::new())
                }

                pub fn new_with_initial_state(
                    rules: Vec<RuleFn<T>>,
                    populate_action: ActionCreationFn<T>,
                    hooks_on_accepted: Vec<HookFn<E>>,
                    hooks_on_rejected: Vec<HookFn<E>>,
                    hooks_after_commit: Vec<HookWithouActionFn<E>>,
                    state: GameState,
                ) -> Self {
                    let action = state.into_action();
                    let mut world = GameWorld {
                        action,
                        populate_action,
                        state: GameState::new(),
                        rules,
                        pending_actions: VecDeque::new(),
                        follow_on_current: VecDeque::new(),
                        follow_on_accepted: VecDeque::new(),
                        follow_on_rejected: VecDeque::new(),
                        hooks_on_accepted,
                        hooks_on_rejected,
                        hooks_after_commit,
                        events_queue: VecDeque::new(),
                        $(
                            [<spatial_ $spatial_type:lower>]: RTree::new(),
                        )*
                    };
                    world.process_initial_state();
                    world
                }

                fn process_initial_state(&mut self) {
                    for hook in &self.hooks_on_accepted {
                        hook(&mut self.events_queue, &self.action, &self.state, $(&self.[<spatial_ $spatial_type:lower>],)*);
                    }

                    $(
                        for (&id, &[<$spatial_type:lower>]) in &self.action.updates.[<$spatial_type:lower>] {
                            if let Some(&[<old_ $spatial_type:lower>]) = self.state.[<get_ $spatial_type:lower>](id) {
                                let [<old_ $spatial_type:lower _tree_object>] = [<$spatial_type TreeObject>] {
                                    index: [<old_ $spatial_type:lower>],
                                    entity_at: id,
                                };

                                self.[<spatial_ $spatial_type:lower>].remove(&[<old_ $spatial_type:lower _tree_object>]);
                            }

                            let [<new_ $spatial_type:lower _tree_object>] = [<$spatial_type TreeObject>] {
                                index: [<$spatial_type:lower>],
                                entity_at: id,
                            };

                            self.[<spatial_ $spatial_type:lower>].insert([<new_ $spatial_type:lower _tree_object>]);

                            #[cfg(feature = "debug_rtrees")]
                            { println!("{:#?}", self.[<spatial_ $spatial_type:lower>]); }
                        }
                    )*

                    self.state.commit_action(&mut self.action);
                }

                pub fn enqueue_action(&mut self, action: T) {
                    self.pending_actions.push_back(action);
                }

                pub fn process_actions(&mut self) {
                    while let Some(action_type) = self.pending_actions.pop_front() {
                        #[cfg(feature = "debug_actions")]
                        { println!("Found an action: {:#?}", action_type); }
                        (self.populate_action)(action_type, &self.state, &mut self.action, $(&self.[<spatial_ $spatial_type:lower>],)*);
                        #[cfg(feature = "debug_state")]
                        { println!("GameState before commiting is: {:#?}", self.state); }

                        let mut accepted = true;

                        for rule in &self.rules {
                            let (action_status, rule_status, mut reactions) = rule(
                                &self.action,
                                &self.state,
                                $(
                                    &self.[<spatial_ $spatial_type:lower>],
                                )*
                            );

                            for a in reactions.drain(..) {
                                self.follow_on_current.push_back(a);
                            }

                            if action_status == ActionStatus::Reject {
                                accepted = false;

                                for a in self.follow_on_current.drain(..) {
                                    self.follow_on_rejected.push_back(a);
                                }
                            } else {
                                for a in self.follow_on_current.drain(..) {
                                    self.follow_on_accepted.push_back(a);
                                }
                            }

                            if rule_status == RuleStatus::StopChecking {
                                break;
                            }
                        }

                        if accepted {
                            #[cfg(feature = "debug_actions")]
                            { println!("Action accepted"); }
                            for hook in &self.hooks_on_accepted {
                                hook(&mut self.events_queue, &self.action, &self.state, $(&self.[<spatial_ $spatial_type:lower>],)*);
                            }

                            $(
                                for (&id, &[<$spatial_type:lower>]) in &self.action.updates.[<$spatial_type:lower>] {
                                    if let Some(&[<old_ $spatial_type:lower>]) = self.state.[<get_ $spatial_type:lower>](id) {
                                        let [<old_ $spatial_type:lower _tree_object>] = [<$spatial_type TreeObject>] {
                                            index: [<old_ $spatial_type:lower>],
                                            entity_at: id,
                                        };

                                        self.[<spatial_ $spatial_type:lower>].remove(&[<old_ $spatial_type:lower _tree_object>]);
                                    }

                                    let [<new_ $spatial_type:lower _tree_object>] = [<$spatial_type TreeObject>] {
                                        index: [<$spatial_type:lower>],
                                        entity_at: id,
                                    };

                                    self.[<spatial_ $spatial_type:lower>].insert([<new_ $spatial_type:lower _tree_object>]);

                                    #[cfg(feature = "debug_rtrees")]
                                    { println!("{:#?}", self.[<spatial_ $spatial_type:lower>]); }
                                }

                                for &id in &self.action.removals.[<$spatial_type:lower>] {
                                    if let Some(&[<old_ $spatial_type:lower>]) = self.state.[<get_ $spatial_type:lower>](id) {
                                        let [<old_ $spatial_type:lower _tree_object>] = [<$spatial_type TreeObject>] {
                                            index: [<old_ $spatial_type:lower>],
                                            entity_at: id,
                                        };

                                        self.[<spatial_ $spatial_type:lower>].remove(&[<old_ $spatial_type:lower _tree_object>]);
                                    }

                                    #[cfg(feature = "debug_rtrees")]
                                    { println!("{:#?}", self.[<spatial_ $spatial_type:lower>]); }
                                }
                            )*

                            self.state.commit_action(&mut self.action);

                            for a in self.follow_on_accepted.drain(..) {
                                self.pending_actions.push_back(a);
                            }
                        } else {
                            #[cfg(feature = "debug_actions")]
                            { println!("Action rejected"); }
                            for hook in &self.hooks_on_rejected {
                                hook(&mut self.events_queue, &self.action, &self.state, $(&self.[<spatial_ $spatial_type:lower>],)*);
                            }

                            self.action.clear();

                            for a in self.follow_on_rejected.drain(..) {
                                self.pending_actions.push_back(a);
                            }
                        }

                        for hook in &self.hooks_after_commit {
                            hook(&mut self.events_queue, &self.state, $(&self.[<spatial_ $spatial_type:lower>],)*);
                        }

                        #[cfg(feature = "debug_state")]
                        { println!("GameState after commiting is: {:#?}", self.state); }
                    }
                }
            }

            $(
                #[derive(Debug, PartialEq)]
                pub struct [<$spatial_type TreeObject>] {
                    pub index: $spatial_type,
                    pub entity_at: $index_type,
                }

                impl RTreeObject for [<$spatial_type TreeObject>] {
                    type Envelope = AABB<$spatial_type>;

                    fn envelope(&self) -> Self::Envelope {
                        AABB::from_point(self.index)
                    }
                }

                impl PointDistance for [<$spatial_type TreeObject>] {
                    fn distance_2(&self, point: &<Self::Envelope as Envelope>::Point,) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
                        point.distance_2(&self.index)
                    }
                }
            )*
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
