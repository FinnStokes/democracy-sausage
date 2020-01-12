#[derive(Clone, Debug)]
pub struct Rectangle([f64; 4]);

impl Rectangle {
    #[inline]
    pub fn new(pos: [f64; 2], size: [f64; 2]) -> Rectangle {
        Rectangle([pos[0], pos[1], size[0], size[1]])
    }
    
    #[inline]
    pub fn centered(pos: [f64; 2], size: [f64; 2]) -> Rectangle {
        Rectangle([pos[0] - size[0] / 2.0, pos[1] - size[1] / 2.0, size[0], size[1]])
    }

    #[inline]
    pub fn intersect_point(&self, point: [f64; 2]) -> bool {
        point[0] > self.0[0] && point[0] < self.0[0] + self.0[2]
            && point[1] > self.0[1] && point[1] < self.0[1] + self.0[3]
    }

    #[inline]
    pub fn intersect_rect(&self, rect: Rectangle) -> bool {
        rect.0[0] + rect.0[2] > self.0[0] && rect.0[0] < self.0[0] + self.0[2]
            && rect.0[1] + rect.0[3] > self.0[1] && rect.0[1] < self.0[1] + self.0[3]
    }

    #[inline]
    pub fn centre(&self) ->[f64; 2] {
        [self.0[0] + self.0[2] / 2.0, self.0[1] + self.0[3] / 2.0]
    }

    pub fn as_floats(self) -> [f64; 4] {
        self.0
    }
}
