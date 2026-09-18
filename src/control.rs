use std::time::{Duration, Instant};

pub const PERIOD: Duration = Duration::from_millis(500);

pub struct Control {
    initial: i32,
    target: i32,
    paused: bool,
    next_check: Instant,
}

impl Control {
    pub fn new(initial: i32, target: i32, now: Instant) -> Self {
        Self {
            initial,
            target,
            paused: false,
            next_check: now,
        }
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    // The operation validates and updates memory; None means it was cancelled
    pub fn step(
        &mut self,
        now: Instant,
        toggle: bool,
        apply: impl FnOnce(i32) -> Result<Option<bool>, String>,
    ) -> Result<Option<bool>, String> {
        if !toggle && (self.paused || now < self.next_check) {
            return Ok(None);
        }
        let target = if toggle && !self.paused {
            self.initial
        } else {
            self.target
        };
        let Some(changed) = apply(target)? else {
            return Ok(None);
        };
        if toggle {
            self.paused = !self.paused;
        }
        self.next_check = now + PERIOD;
        Ok(Some(changed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodic_correction_pause_and_resume_preserve_startup_value() {
        for initial in [30, 60, 120] {
            let start = Instant::now();
            let mut control = Control::new(initial, 90, start);
            let mut fps = initial;
            let mut apply = |target| {
                let changed = fps != target;
                fps = target;
                Ok(Some(changed))
            };
            assert_eq!(control.step(start, false, &mut apply).unwrap(), Some(true));
            assert_eq!(fps, 90);
            control
                .step(start + PERIOD / 2, false, |_| panic!("early access"))
                .unwrap();
            fps = 30;
            control
                .step(start + PERIOD, false, |target| {
                    assert_eq!(fps, 30);
                    fps = target;
                    Ok(Some(true))
                })
                .unwrap();
            assert_eq!(fps, 90);
            control
                .step(start + PERIOD, true, |target| {
                    fps = target;
                    Ok(Some(true))
                })
                .unwrap();
            assert!(control.paused());
            assert_eq!(fps, initial);
            fps = 60;
            control
                .step(start + PERIOD * 10, false, |_| {
                    panic!("paused memory access")
                })
                .unwrap();
            assert_eq!(fps, 60);
            control
                .step(start + PERIOD * 10, true, |target| {
                    fps = target;
                    Ok(Some(true))
                })
                .unwrap();
            assert!(!control.paused());
            assert_eq!(fps, 90);
            control
                .step(start + PERIOD * 11, true, |target| {
                    assert_eq!(target, initial);
                    Ok(Some(false))
                })
                .unwrap();
            assert!(control.paused());
        }
    }

    #[test]
    fn failed_or_cancelled_updates_do_not_commit_a_toggle() {
        let now = Instant::now();
        let mut control = Control::new(60, 120, now);
        for paused in [false, true] {
            assert_eq!(control.paused(), paused);
            for failure in ["invalid FPS", "locator changed", "write failed"] {
                assert_eq!(
                    control.step(now, true, |_| Err(failure.into())),
                    Err(failure.into())
                );
                assert_eq!(control.paused(), paused);
            }
            assert_eq!(control.step(now, true, |_| Ok(None)).unwrap(), None);
            assert_eq!(control.paused(), paused);
            control.step(now, true, |_| Ok(Some(false))).unwrap();
        }
    }
}
