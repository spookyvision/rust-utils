pub mod stick {

    #[derive(Debug)]
    pub struct Axis {
        min: u16,
        max: u16,
        mid: u16,
        done: bool,
    }

    impl Axis {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn set_mid(&mut self, val: u16) {
            self.mid = val;
        }

        pub fn feed(&mut self, val: u16) {
            self.min = self.min.min(val);
            self.max = self.max.max(val);
        }

        pub fn eval(&mut self, val: u16) -> Option<f32> {
            if !self.done {
                return None;
            }

            // instead of clamping later we update min/max
            self.min = self.min.min(val);
            self.max = self.max.max(val);

            let res = if val < self.mid {
                let nom = (val - self.min) as f32;
                let denom = (self.mid - self.min) as f32;
                nom / denom - 1.
            } else {
                let nom = (val - self.mid) as f32;
                let denom = (self.max - self.mid) as f32;
                nom / denom
            };

            Some(res)
        }

        pub fn done(&self) -> bool {
            self.done
        }

        pub fn set_done(&mut self, done: bool) {
            self.done = done;
        }
    }

    impl Default for Axis {
        fn default() -> Self {
            let min = u16::MAX;
            let max = u16::MIN;

            Self {
                min,
                max,
                mid: 0,
                done: false,
            }
        }
    }

    #[derive(Default, Debug)]
    pub struct Stick {
        x: Axis,
        y: Axis,
    }

    impl Stick {
        pub fn x_mut(&mut self) -> &mut Axis {
            &mut self.x
        }

        pub fn y_mut(&mut self) -> &mut Axis {
            &mut self.y
        }
    }
}
