use std::collections::HashMap;
use std::hash::Hash;

use rand::seq::IndexedRandom;

pub trait Environment {
    type State: Copy + Eq + Hash;
    type Action: Copy + Eq + Hash;
    fn actions(state: &Self::State) -> Vec<Self::Action>;
    fn step(state: &Self::State, action: &Self::Action) -> (Option<Self::State>, f32);
    fn new() -> Self::State;
}

pub trait Policy<E: Environment> {
    fn choose_action(&self, state: &E::State) -> E::Action;
    fn improve(
        &mut self,
        state: &E::State,
        action: E::Action,
        reward: f32,
        next_state: Option<&E::State>,
    );
    fn on_episode_increment(&mut self) {}
}

pub struct QLearning;

impl QLearning {
    pub fn train<E: Environment>(
        policy: &mut impl Policy<E>,
        num_training_episodes: usize,
        max_steps: Option<usize>,
    ) {
        for _ in 0..num_training_episodes {
            QLearning::one_episode(policy, max_steps);
            policy.on_episode_increment();
        }
    }

    fn one_episode<E: Environment>(policy: &mut impl Policy<E>, max_steps: Option<usize>) {
        let mut state = E::new();

        if let Some(m) = max_steps {
            for _ in 0..m {
                if let Some(next_state) = QLearning::choose_and_improve(policy, state) {
                    state = next_state;
                } else {
                    break;
                }
            }
        } else {
            loop {
                if let Some(next_state) = QLearning::choose_and_improve(policy, state) {
                    state = next_state;
                } else {
                    break;
                }
            }
        }
    }

    fn choose_and_improve<E: Environment>(
        policy: &mut impl Policy<E>,
        state: E::State,
    ) -> Option<E::State> {
        let action = policy.choose_action(&state);

        let (next_state, reward) = E::step(&state, &action);
        policy.improve(&state, action, reward, next_state.as_ref());
        next_state
    }
}

pub struct GreedyPolicy<E: Environment> {
    qtable: HashMap<(E::State, E::Action), f32>,
    learning_rate: f32,
    gamma: f32,
}

impl<E: Environment> Policy<E> for GreedyPolicy<E> {
    fn choose_action(&self, state: &<E as Environment>::State) -> <E as Environment>::Action {
        let actions = E::actions(state);
        *actions.iter()
            .max_by(|&a, &b|
                self.qtable.get(&(*state, *a))
                    .unwrap_or(&0f32)
                    .total_cmp(self.qtable.get(&(*state, *b))
                    .unwrap_or(&0f32))
            )
            .expect(
            "The way it is implemented now, there should always be possible actions (might be bad)",
        )
    }
    fn improve(
        &mut self,
        state: &<E as Environment>::State,
        action: <E as Environment>::Action,
        reward: f32,
        next_state: Option<&<E as Environment>::State>,
    ) {
        let former_value = *self.qtable.get(&(*state, action)).unwrap_or(&0f32);
        let target = match next_state {
            Some(next_state) => {
                reward
                    + self.gamma
                        * self
                            .qtable
                            .get(&(*next_state, self.choose_action(next_state)))
                            .unwrap_or(&0f32)
            }
            None => 0f32,
        };
        self.qtable.insert(
            (*state, action),
            former_value + self.learning_rate * (target - former_value),
        );
    }
}

pub struct EpsilonGreedyPolicy<E: Environment> {
    greedy_policy: GreedyPolicy<E>,
    min_epsilon: f32,
    max_epsilon: f32,
    decay_rate: f32,
    episode: usize,
}

impl<E: Environment> EpsilonGreedyPolicy<E> {
    fn epsilon(&self) -> f32 {
        self.min_epsilon
            + (self.max_epsilon - self.min_epsilon) * (-self.decay_rate * self.episode as f32).exp()
    }
}

impl<E: Environment> Policy<E> for EpsilonGreedyPolicy<E> {
    fn choose_action(&self, state: &<E as Environment>::State) -> <E as Environment>::Action {
        let action: E::Action;
        if rand::random_range(0f32..1f32) < self.epsilon() {
            action = *E::actions(state).choose(&mut rand::rng()).expect(
                "The way it is implemented now, there should always be possible actions (might be bad)",
            );
        } else {
            action = self.greedy_policy.choose_action(state);
        }

        action
    }

    fn improve(
        &mut self,
        state: &<E as Environment>::State,
        action: <E as Environment>::Action,
        reward: f32,
        next_state: Option<&<E as Environment>::State>,
    ) {
        self.greedy_policy
            .improve(state, action, reward, next_state);
    }

    fn on_episode_increment(&mut self) {
        self.episode += 1;
    }
}
