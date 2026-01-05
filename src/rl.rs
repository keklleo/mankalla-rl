pub trait Environment {
    type State;
    type Action;
    type Reward;
    fn actions(state: &Self::State) -> Vec<Self::Action>;
    fn step(state: Self::State, action: Self::Action) -> (Option<Self::State>, Self::Reward);
}
