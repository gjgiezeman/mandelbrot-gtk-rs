pub struct Preset {
    cx: f64,
    cy: f64,
    zoom: i32,
    iter_depth: i32,
}

impl Preset {
    fn new(cx: f64, cy: f64, zoom: i32, iter_depth: i32) -> Preset {
        Preset {
            cx,
            cy,
            zoom,
            iter_depth,
        }
    }
    pub fn cx(&self) -> f64 {
        self.cx
    }
    pub fn cy(&self) -> f64 {
        self.cy
    }
    pub fn zoom(&self) -> f64 {
        self.zoom as f64
    }
    pub fn iter_depth(&self) -> f64 {
        self.iter_depth as f64
    }
}

pub struct Presets {
    names: Vec<&'static str>,
    values: Vec<Preset>,
}

impl Presets {
    pub fn new() -> Presets {
        let names = vec!["Initial", "Flamenco", "Spiral"];
        let values = vec![
            Preset::new(0.0, 0.0, 0, 100),
            Preset::new(-1.7665088674631104, 0.04172334239500609, 750, 1000),
            Preset::new(-0.8099833738092991, 0.17004289101216644, 500, 1000),
        ];
        assert_eq!(names.len(), values.len());
        Presets { names, values }
    }
    pub fn names(&self) -> &[&str] {
        self.names.as_slice()
    }
    pub fn len(&self) -> usize {
        self.names.len()
    }
    pub fn get(&self, i: usize) -> &Preset {
        assert!(i < self.len());
        &self.values[i]
    }
}
