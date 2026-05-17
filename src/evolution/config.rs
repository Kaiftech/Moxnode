/// Toggleable evolution subsystems — all off by default for stability.
#[derive(Debug, Clone, Copy)]
pub struct EvolutionConfig {
    pub internet: bool,
    pub memory_decay: bool,
    pub society: bool,
    pub habits: bool,
    pub priorities: bool,
    pub dreams: bool,
    pub rituals: bool,
    pub expression: bool,
    pub leaps: bool,
}

impl EvolutionConfig {
    pub fn none() -> Self {
        Self {
            internet: false,
            memory_decay: false,
            society: false,
            habits: false,
            priorities: false,
            dreams: false,
            rituals: false,
            expression: false,
            leaps: false,
        }
    }

    pub fn all() -> Self {
        Self {
            internet: true,
            memory_decay: true,
            society: true,
            habits: true,
            priorities: true,
            dreams: true,
            rituals: true,
            expression: true,
            leaps: true,
        }
    }

    pub fn any(&self) -> bool {
        self.internet
            || self.memory_decay
            || self.society
            || self.habits
            || self.priorities
            || self.dreams
            || self.rituals
            || self.expression
            || self.leaps
    }

    pub fn from_cli(
        evolution: bool,
        internet: bool,
        memory_decay: bool,
        society: bool,
        habits: bool,
        priorities: bool,
        dreams: bool,
        rituals: bool,
        expression: bool,
        leaps: bool,
    ) -> Self {
        if evolution {
            return Self::all();
        }
        Self {
            internet,
            memory_decay,
            society,
            habits,
            priorities,
            dreams,
            rituals,
            expression,
            leaps,
        }
    }
}
