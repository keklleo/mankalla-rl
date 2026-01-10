use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

use rand::seq::IndexedRandom;

pub trait Environment {
    type State: Copy;
    type ActionRelevantState: From<Self::State> + Copy + Eq + Hash + Serialize + Deserialize;
    type Action: Copy + Eq + Hash + Serialize + Deserialize;
    fn actions(state: &Self::ActionRelevantState) -> Vec<Self::Action>;
    fn step(state: &Self::State, action: &Self::Action) -> (Option<Self::State>, f32);
    fn new() -> Self::State;
}

pub trait Policy<E: Environment> {
    fn choose_action(&self, state: E::ActionRelevantState) -> E::Action;
    fn improve(
        &mut self,
        state: &E::ActionRelevantState,
        action: E::Action,
        reward: f32,
        next_state: Option<&E::State>,
    );
    fn on_episode_increment(&mut self) {}
}

pub trait Serialize {
    fn serialize(&self) -> String;
}

pub trait Deserialize {
    fn deserialize(input: &str) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct DeserializeError;

impl Error for DeserializeError {}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error deserializing input")
    }
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
        let action = policy.choose_action(state.into());

        let (next_state, reward) = E::step(&state, &action);
        policy.improve(&state.into(), action, reward, next_state.as_ref());
        next_state
    }
}

pub struct GreedyPolicy<E: Environment> {
    qtable: HashMap<(E::ActionRelevantState, E::Action), f32>,
    learning_rate: f32,
    gamma: f32,
}

impl<E: Environment> GreedyPolicy<E> {
    pub fn new(learning_rate: f32, gamma: f32) -> Self {
        GreedyPolicy {
            qtable: HashMap::new(),
            learning_rate,
            gamma,
        }
    }
}

impl<E: Environment> Policy<E> for GreedyPolicy<E> {
    fn choose_action(&self, state: E::ActionRelevantState) -> E::Action {
        let actions = E::actions(&state);
        *actions.iter()
            .max_by(|&a, &b|
                self.qtable.get(&(state, *a))
                    .unwrap_or(&0f32)
                    .total_cmp(self.qtable.get(&(state, *b))
                    .unwrap_or(&0f32))
            )
            .expect(
            "The way it is implemented now, there should always be possible actions (might be bad)",
        )
    }
    fn improve(
        &mut self,
        state: &E::ActionRelevantState,
        action: E::Action,
        reward: f32,
        next_state: Option<&E::State>,
    ) {
        let former_value = *self.qtable.get(&(*state, action)).unwrap_or(&0f32);
        let target = match next_state {
            Some(next_state) => {
                reward
                    + self.gamma
                        * self
                            .qtable
                            .get(&(
                                (*next_state).into(),
                                self.choose_action((*next_state).into()),
                            ))
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

impl<E: Environment> Serialize for GreedyPolicy<E> {
    fn serialize(&self) -> String {
        format!("{};{}\n", self.gamma, self.learning_rate)
            + self
                .qtable
                .iter()
                .map(|((state, action), value)| {
                    format!("{};{};{}\n", state.serialize(), action.serialize(), value)
                })
                .reduce(|a, b| a + b.as_str())
                .unwrap_or(String::new())
                .as_str()
    }
}

impl<E: Environment> Deserialize for GreedyPolicy<E> {
    fn deserialize(input: &str) -> Result<Self, DeserializeError> {
        let mut lines = input.lines();

        let mut parameters = match lines.next() {
            Some(s) => s.split(';').map(|a| a.parse::<f32>()),
            _ => return Err(DeserializeError),
        };
        let gamma = match parameters.next() {
            Some(Ok(f)) => f,
            _ => return Err(DeserializeError),
        };
        let learning_rate = match parameters.next() {
            Some(Ok(f)) => f,
            _ => return Err(DeserializeError),
        };
        if parameters.next() != None {
            return Err(DeserializeError);
        }

        let mut qtable = HashMap::<(E::ActionRelevantState, E::Action), f32>::new();
        for line in lines {
            let mut parts = line.split(';');
            let state = match parts.next() {
                Some(s) => E::ActionRelevantState::deserialize(s)?,
                _ => return Err(DeserializeError),
            };
            let action = match parts.next() {
                Some(a) => E::Action::deserialize(a)?,
                _ => return Err(DeserializeError),
            };
            let value_result = match parts.next() {
                Some(v) => v.parse::<f32>(),
                _ => return Err(DeserializeError),
            };
            let value = match value_result {
                Ok(v) => v,
                _ => return Err(DeserializeError),
            };
            if parts.next() != None {
                return Err(DeserializeError);
            }

            qtable.insert((state, action), value);
        }

        Ok(GreedyPolicy::<E> {
            qtable,
            gamma,
            learning_rate,
        })
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
    pub fn new(
        learning_rate: f32,
        gamma: f32,
        max_epsilon: f32,
        min_epsilon: f32,
        decay_rate: f32,
    ) -> Self {
        EpsilonGreedyPolicy {
            greedy_policy: GreedyPolicy::new(learning_rate, gamma),
            min_epsilon,
            max_epsilon,
            decay_rate,
            episode: 0,
        }
    }

    fn epsilon(&self) -> f32 {
        self.min_epsilon
            + (self.max_epsilon - self.min_epsilon) * (-self.decay_rate * self.episode as f32).exp()
    }
}

impl<E: Environment> Policy<E> for EpsilonGreedyPolicy<E> {
    fn choose_action(&self, state: E::ActionRelevantState) -> E::Action {
        let action: E::Action;
        if rand::random_range(0f32..1f32) < self.epsilon() {
            action = *E::actions(&state).choose(&mut rand::rng()).expect(
                "The way it is implemented now, there should always be possible actions (might be bad)",
            );
        } else {
            action = self.greedy_policy.choose_action(state);
        }

        action
    }

    fn improve(
        &mut self,
        state: &E::ActionRelevantState,
        action: E::Action,
        reward: f32,
        next_state: Option<&E::State>,
    ) {
        self.greedy_policy
            .improve(state, action, reward, next_state);
    }

    fn on_episode_increment(&mut self) {
        self.episode += 1;
    }
}

impl<E: Environment> Serialize for EpsilonGreedyPolicy<E> {
    fn serialize(&self) -> String {
        format!(
            "{};{};{};{}\n",
            self.min_epsilon, self.max_epsilon, self.decay_rate, self.episode
        ) + self.greedy_policy.serialize().as_str()
    }
}

impl<E: Environment> Deserialize for EpsilonGreedyPolicy<E> {
    fn deserialize(input: &str) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let (parts, rest) = match input.split_once('\n') {
            Some(s) => s,
            _ => return Err(DeserializeError),
        };
        let mut parts = parts.split(';').map(|a| a.parse::<f32>());
        let min_epsilon = match parts.next() {
            Some(Ok(m)) => m,
            _ => return Err(DeserializeError),
        };
        let max_epsilon = match parts.next() {
            Some(Ok(m)) => m,
            _ => return Err(DeserializeError),
        };
        let decay_rate = match parts.next() {
            Some(Ok(d)) => d,
            _ => return Err(DeserializeError),
        };
        let episode = match parts.next() {
            Some(Ok(e)) => e,
            _ => return Err(DeserializeError),
        };
        if parts.next() != None {
            return Err(DeserializeError);
        }

        Ok(EpsilonGreedyPolicy::<E> {
            greedy_policy: GreedyPolicy::<E>::deserialize(rest)?,
            min_epsilon,
            max_epsilon,
            decay_rate,
            episode: episode as usize,
        })
    }
}
