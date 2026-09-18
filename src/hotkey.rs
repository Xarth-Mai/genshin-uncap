#[derive(Default)]
pub struct F10Edge {
    armed: bool,
}

impl F10Edge {
    pub fn pressed(&mut self, game_foreground: bool, down: bool, modified: bool) -> bool {
        if !game_foreground || modified {
            self.armed = false;
            return false;
        }
        if !down {
            self.armed = true;
            return false;
        }
        std::mem::replace(&mut self.armed, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_fresh_unmodified_foreground_presses_toggle() {
        let mut key = F10Edge::default();
        assert!(!key.pressed(true, true, false));
        assert!(!key.pressed(true, false, false));
        assert!(key.pressed(true, true, false));
        assert!(!key.pressed(true, true, false));
        assert!(!key.pressed(false, false, false));
        assert!(!key.pressed(false, true, false));
        assert!(!key.pressed(true, true, false));
        assert!(!key.pressed(true, false, false));
        assert!(!key.pressed(true, true, true));
        assert!(!key.pressed(true, true, false));
        assert!(!key.pressed(true, false, false));
        assert!(key.pressed(true, true, false));
    }
}
