use std::collections::HashMap;

pub trait Environment {
    type State;
    type Action;
    type Reward;
    fn actions(state: &Self::State) -> Vec<Self::Action>;
    fn step(state: Self::State, action: Self::Action) -> (Option<Self::State>, Self::Reward);
}

pub trait ToSaveState: Environment {
    type SaveState;
    fn to_save_state(state: Self::State) -> Self::SaveState;
}

trait Policy<E: Environment> {
    type ActionEvaluator;
    fn choose_action(action_evaluator: Self::ActionEvaluator, state: E::State) -> E::Action;
    fn train(
        action_evaluator: Self::ActionEvaluator,
        num_training_episodes: usize,
        max_steps: usize,
    ) -> Self::ActionEvaluator;
}

trait SavePolicy<T: ToSaveState>: Policy<T> {
    fn save_policy(policy: Self::ActionEvaluator);
}

enum PolicyKind {
    Greedy,
    EpsilonGreedy,
}

struct QLearning<E: ToSaveState> {
    qtable: HashMap<E::SaveState, isize>,
}
