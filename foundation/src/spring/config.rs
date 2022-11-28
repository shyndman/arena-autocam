#[derive(Clone)]
pub struct SpringConfig {
    pub tension: f64,
    pub friction: f64,
    pub mass: f64,
    pub precision: Option<f64>,
    pub initial_velocity: Option<f64>,
    pub max_velocity: Option<f64>,
    pub clamp: bool,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            tension: 170.0,
            friction: 26.0,
            mass: 1.0,
            precision: None,
            initial_velocity: None,
            max_velocity: None,
            clamp: true,
        }
    }
}

impl SpringConfig {
    pub fn initial_velocity(&self) -> f64 {
        self.initial_velocity.unwrap_or(0.0)
    }

    pub fn max_velocity(&self) -> f64 {
        self.max_velocity.unwrap_or(f64::MAX)
    }

    /// Configuration presets

    pub fn gentle() -> Self {
        Self {
            tension: 120.0,
            friction: 14.0,
            ..Default::default()
        }
    }

    pub fn wobbly() -> Self {
        Self {
            tension: 180.0,
            friction: 12.0,
            ..Default::default()
        }
    }

    pub fn stiff() -> Self {
        Self {
            tension: 210.0,
            friction: 20.0,
            ..Default::default()
        }
    }

    pub fn slow() -> Self {
        Self {
            tension: 280.0,
            friction: 60.0,
            ..Default::default()
        }
    }

    pub fn molasses() -> Self {
        Self {
            tension: 280.0,
            friction: 120.0,
            ..Default::default()
        }
    }
}
