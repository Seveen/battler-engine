pub extern crate paste;

#[macro_export]
macro_rules! register_components {
    ( $index_type:ty, $( $component_type:ty ),* ) => {
        $crate::paste::paste! {
            #[derive(Debug)]
            struct State {
                $(
                    [<$component_type:lower>]: HashMap<$index_type, $component_type>,
                )*
            }

            impl State {
                fn new() -> Self {
                    State {
                        $(
                            [<$component_type:lower>]: HashMap::new(),
                        )*
                    }
                }

                fn clear(&mut self) {
                    $(
                        self.[<$component_type:lower>].clear();
                    )*
                }

                $(
                    fn [<get_ $component_type:lower>](&self, id: $index_type) -> Option<$component_type> {
                        self.[<$component_type:lower>].get(&id).copied()
                    }
                )*

                fn commit_action(&mut self, action: &mut Action) {
                    $(
                        for (id, value) in action.updates.[<$component_type:lower>].drain() {
                            self.[<$component_type:lower>].insert(id, value);
                        }
                        for id in action.removals.[<$component_type:lower>].drain() {
                            self.[<$component_type:lower>].remove(&id);
                        }
                    )*
                }
            }

            #[derive(Debug)]
            struct RemovedComponents {
                $(
                    [<$component_type:lower>]: HashSet<$index_type>,
                )*
            }

            impl RemovedComponents {
                fn new() -> Self {
                    RemovedComponents {
                    $(
                        [<$component_type:lower>]: HashSet::new(),
                    )*
                    }
                }

                fn clear(&mut self) {
                    $(
                        self.[<$component_type:lower>].clear();
                    )*
                }
            }

            #[derive(Debug)]
            struct Action {
                updates: State,
                removals: RemovedComponents,
            }

            impl Action {
                fn new() -> Self {
                    Action {
                        updates: State::new(),
                        removals: RemovedComponents::new(),
                    }
                }

                fn clear(&mut self) {
                    self.updates.clear();
                    self.removals.clear();
                }

                $(
                    fn [<insert_ $component_type:lower>](&mut self, id: $index_type, value: $component_type) {
                        self.updates.[<$component_type:lower>].insert(id, value);
                    }

                    fn [<remove_ $component_type:lower>](&mut self, id: $index_type) {
                        self.removals.[<$component_type:lower>].insert(id);
                    }
                )*

                fn remove_all(&mut self, id: EntityId) {
                    $(
                        self.[<remove_ $component_type:lower>](id);
                    )*
                }
            }

            struct FutureState {
                state: State,
                action: Action,
            }

            impl FutureState {
                $(
                    fn [<get_ $component_type:lower>](&self, id: $index_type) -> Option<$component_type> {
                        if let Some(value) = self.action.updates.[<get_ $component_type:lower>](id) {
                            return Some(value);
                        }
                        if self.action.removals.[<$component_type:lower>].contains(&id) {
                            return None;
                        }
                        self.state.[<get_ $component_type:lower>](id)
                    }
                )*
            }

            #[derive(Debug, PartialEq, Eq)]
            enum ActionStatus {
                Accept,
                Reject,
            }

            #[derive(Debug, PartialEq, Eq)]
            enum RuleStatus {
                KeepChecking,
                StopChecking,
            }

            type RuleFn<T> = fn(
                &Action,
                &State,
            ) -> (ActionStatus, RuleStatus, Vec<T>);

            type ActionCreationFn<T> = fn(
                T,
                &State,
                &mut Action,
            );

            struct Battle<T> {
                populate_action: ActionCreationFn<T>,
                action: Action,
                state: State,
                rules: Vec<RuleFn<T>>,
                pending_actions: VecDeque<T>,
                follow_on_current: VecDeque<T>,
                follow_on_accepted: VecDeque<T>,
                follow_on_rejected: VecDeque<T>,
            }

            impl<T: Debug> Battle<T> {
                fn new(rules: Vec<RuleFn<T>>, populate_action: ActionCreationFn<T>) -> Self {
                    Battle::new_with_initial_state(rules, populate_action, State::new())
                }

                fn new_with_initial_state(rules: Vec<RuleFn<T>>, populate_action: ActionCreationFn<T>, state: State) -> Self {
                    Battle {
                        action: Action::new(),
                        populate_action,
                        state,
                        rules,
                        pending_actions: VecDeque::new(),
                        follow_on_current: VecDeque::new(),
                        follow_on_accepted: VecDeque::new(),
                        follow_on_rejected: VecDeque::new(),
                    }
                }

                fn enqueue_action(&mut self, action: T) {
                    self.pending_actions.push_back(action);
                }

                fn process_actions(&mut self) {
                    while let Some(action_type) = self.pending_actions.pop_front() {
                        println!("Found an action: {:?}", action_type);
                        (self.populate_action)(action_type, &self.state, &mut self.action);
                        println!("State before commiting is: {:?}", self.state);

                        let mut accepted = true;

                        for rule in &self.rules {
                            let (action_status, rule_status, mut reactions) = rule(
                                &self.action,
                                &self.state,
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

                            if accepted {
                                self.state.commit_action(&mut self.action);

                                for a in self.follow_on_accepted.drain(..) {
                                    self.pending_actions.push_back(a);
                                }
                            } else {
                                self.action.clear();

                                for a in self.follow_on_rejected.drain(..) {
                                    self.pending_actions.push_back(a);
                                }
                            }
                        }
                        println!("State after commiting is: {:?}", self.state);
                    }
                }
            }
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
