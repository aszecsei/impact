#[derive(Debug, Clone)]
pub struct RectSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Default, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    /// Performs a lexicographic compare on (rect short side, rect long side).
    pub fn compare_rect_short_side(&self, b: &Rect) -> std::cmp::Ordering {
        let smaller_side_a = std::cmp::min(self.width, self.height);
        let smaller_side_b = std::cmp::min(b.width, b.height);
        if smaller_side_a != smaller_side_b {
            return smaller_side_a.cmp(&smaller_side_b);
        }

        // Tie-break on larger side
        let larger_side_a = std::cmp::max(self.width, self.height);
        let larger_side_b = std::cmp::max(b.width, b.height);
        return larger_side_a.cmp(&larger_side_b);
    }

    pub fn node_sort_cmp(a: &Rect, b: &Rect) -> std::cmp::Ordering {
        if a.x != b.x {
            return a.x.cmp(&b.x);
        }
        if a.y != b.y {
            return a.y.cmp(&b.y);
        }
        if a.width != b.width {
            return a.width.cmp(&b.width);
        }
        return a.height.cmp(&b.height);
    }

    pub fn is_contained_in(&self, b: &Rect) -> bool {
        self.x >= b.x
            && self.y >= b.y
            && self.x + self.width <= b.x + b.width
            && self.y + self.height <= b.y + b.height
    }
}

pub struct DisjointRectCollection {
    pub rects: Vec<Rect>,
}

impl DisjointRectCollection {
    pub fn new() -> Self {
        Self { rects: vec![] }
    }

    pub fn add(&mut self, r: &Rect) -> bool {
        // Degenerate rectangles are ignored
        if r.width == 0 || r.height == 0 {
            return true;
        }

        if !self.disjoint(r) {
            return false;
        }

        self.rects.push(r.clone());
        true
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }

    pub fn disjoint(&self, r: &Rect) -> bool {
        // Degenerate rectangles are ignored
        if r.width == 0 || r.height == 0 {
            return true;
        }

        for a in &self.rects {
            if !disjoint(a, r) {
                return false;
            }
        }

        true
    }
}

fn disjoint(a: &Rect, b: &Rect) -> bool {
    a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y
}
