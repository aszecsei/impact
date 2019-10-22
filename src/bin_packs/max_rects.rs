use crate::rect::Rect;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone)]
pub enum FreeRectChoiceHeuristic {
    /// BSSF: Positions the rectangle against the short side of a free rectangle into which it fits the best.
    RectBestShortSideFit,
    /// BLSF: Positions the rectangle against the long side of a free rectangle into which it fits the best.
    RectBestLongSideFit,
    /// BAF: Positions the rectangle into the smallest free rect into which it fits.
    RectBestAreaFit,
    /// BL: Does the Tetris placement.
    RectBottomLeftRule,
    /// CP: Choosest the placement where the rectangle touches other rects as much as possible.
    RectContactPointRule,
}

pub struct MaxRectsBinPack {
    bin_width: i32,
    bin_height: i32,
    used_rectangles: Vec<Rect>,
    free_rectangles: Vec<Rect>,
}

impl MaxRectsBinPack {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            bin_width: width,
            bin_height: height,
            used_rectangles: vec![],
            free_rectangles: vec![Rect {
                x: 0,
                y: 0,
                width,
                height,
            }],
        }
    }

    pub fn insert_list(
        &mut self,
        rects: &[Rect],
        rot: bool,
        method: FreeRectChoiceHeuristic,
    ) -> Vec<Rect> {
        let mut dst = vec![];
        let mut rects = Vec::from(rects);

        while rects.len() > 0 {
            let mut best_score_1 = i32::max_value();
            let mut best_score_2 = i32::max_value();
            let mut best_rect_index = -1;
            let mut best_node = Rect::default();

            for (idx, rect) in rects.iter().enumerate() {
                let (new_node, score1, score2) =
                    self.score_rect(rect.width, rect.height, rot, method);

                if score1 < best_score_1 || (score1 == best_score_1 && score2 < best_score_2) {
                    best_score_1 = score1;
                    best_score_2 = score2;
                    best_node = new_node;
                    best_rect_index = idx as i32;
                }
            }

            if best_rect_index == -1 {
                break;
            }

            self.place_rect(&best_node);
            rects.remove(best_rect_index.try_into().unwrap());
            dst.push(best_node);
        }

        dst
    }

    pub fn insert(
        &mut self,
        width: i32,
        height: i32,
        rot: bool,
        method: FreeRectChoiceHeuristic,
    ) -> Rect {
        let (new_node, _, _) = match method {
            FreeRectChoiceHeuristic::RectBestShortSideFit => {
                self.find_position_for_new_node_best_short_side_fit(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectBottomLeftRule => {
                self.find_position_for_new_node_bottom_left(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectContactPointRule => {
                let (a, b) = self.find_position_for_new_node_contact_point(rot, width, height);
                (a, b, 0)
            }
            FreeRectChoiceHeuristic::RectBestLongSideFit => {
                self.find_position_for_new_node_best_long_side_fit(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectBestAreaFit => {
                self.find_position_for_new_node_best_area_fit(rot, width, height)
            }
        };

        if new_node.height == 0 {
            return new_node;
        }

        self.place_rect(&new_node);
        new_node
    }

    pub fn occupancy(&self) -> f32 {
        let mut used_surface_area = 0;
        for rect in &self.used_rectangles {
            used_surface_area += rect.width * rect.height;
        }
        (used_surface_area as f32) / ((self.bin_width * self.bin_height) as f32)
    }

    fn score_rect(
        &self,
        width: i32,
        height: i32,
        rot: bool,
        method: FreeRectChoiceHeuristic,
    ) -> (Rect, i32, i32) {
        let (new_node, mut score1, mut score2) = match method {
            FreeRectChoiceHeuristic::RectBestShortSideFit => {
                self.find_position_for_new_node_best_short_side_fit(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectBottomLeftRule => {
                self.find_position_for_new_node_bottom_left(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectContactPointRule => {
                let (r, s1) = self.find_position_for_new_node_contact_point(rot, width, height);
                // Reverse since we're minimizing, but for contact point score bigger is better
                (r, -s1, i32::max_value())
            }
            FreeRectChoiceHeuristic::RectBestLongSideFit => {
                self.find_position_for_new_node_best_long_side_fit(rot, width, height)
            }
            FreeRectChoiceHeuristic::RectBestAreaFit => {
                self.find_position_for_new_node_best_area_fit(rot, width, height)
            }
        };

        // Cannot fit the current rectangle
        if new_node.height == 0 {
            score1 = i32::max_value();
            score2 = i32::max_value();
        }

        (new_node, score1, score2)
    }

    fn place_rect(&mut self, node: &Rect) {
        let mut num_rectangles_to_process = self.free_rectangles.len();
        let mut i = 0;
        while i < num_rectangles_to_process {
            let r = self.free_rectangles[i].clone();
            if self.split_free_node(&r, node) {
                self.free_rectangles.remove(i);
                num_rectangles_to_process -= 1;
            } else {
                i += 1;
            }
        }

        self.prune_free_list();

        self.used_rectangles.push(node.clone());
    }

    fn contact_point_score_node(&self, x: i32, y: i32, width: i32, height: i32) -> i32 {
        let mut score = 0;

        if x == 0 || x + width == self.bin_width {
            score += height;
        }
        if y == 0 || y + height == self.bin_height {
            score += width;
        }

        for rect in &self.used_rectangles {
            if rect.x == x + width || rect.x + rect.width == x {
                score += common_interval_length(rect.y, rect.y + rect.height, y, y + height);
            }
            if rect.y == y + height || rect.y + rect.height == y {
                score += common_interval_length(rect.x, rect.x + rect.width, x, x + width)
            }
        }

        score
    }
    fn find_position_for_new_node_bottom_left(
        &self,
        rot: bool,
        width: i32,
        height: i32,
    ) -> (Rect, i32, i32) {
        let mut best_node = Rect::default();

        let mut best_y = i32::max_value();
        let mut best_x = i32::max_value();

        for rect in &self.free_rectangles {
            // Try to place the rectangle in upright (non-flipped) orientation
            if rect.width >= width && rect.height >= height {
                let top_side_y = rect.y + height;
                if top_side_y < best_y || (top_side_y == best_y && rect.x < best_x) {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    best_y = top_side_y;
                    best_x = rect.x;
                }
            }
            if rot && rect.width >= height && rect.height >= width {
                let top_side_y = rect.y + width;
                if top_side_y < best_y || (top_side_y == best_y && rect.x < best_x) {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    best_y = top_side_y;
                    best_x = rect.x;
                }
            }
        }

        (best_node, best_y, best_x)
    }
    fn find_position_for_new_node_best_short_side_fit(
        &self,
        rot: bool,
        width: i32,
        height: i32,
    ) -> (Rect, i32, i32) {
        let mut best_node = Rect::default();

        let mut best_short_side_fit = i32::max_value();
        let mut best_long_side_fit = i32::max_value();

        for rect in &self.free_rectangles {
            // Try to place the rectangle in upright (non-flipped) orientation
            if rect.width >= width && rect.height >= height {
                let leftover_horiz = (rect.width - width).abs();
                let leftover_vert = (rect.height - height).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = std::cmp::max(leftover_horiz, leftover_vert);
                if short_side_fit < best_short_side_fit
                    || (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = best_long_side_fit;
                }
            }
            if rot && rect.width >= height && rect.height >= width {
                let leftover_horiz = (rect.width - height).abs();
                let leftover_vert = (rect.height - width).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = std::cmp::max(leftover_horiz, leftover_vert);
                if short_side_fit < best_short_side_fit
                    || (short_side_fit == best_short_side_fit && long_side_fit < best_long_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = best_long_side_fit;
                }
            }
        }

        (best_node, best_short_side_fit, best_long_side_fit)
    }
    fn find_position_for_new_node_best_long_side_fit(
        &self,
        rot: bool,
        width: i32,
        height: i32,
    ) -> (Rect, i32, i32) {
        let mut best_node = Rect::default();

        let mut best_short_side_fit = i32::max_value();
        let mut best_long_side_fit = i32::max_value();

        for rect in &self.free_rectangles {
            // Try to place the rectangle in upright (non-flipped) orientation
            if rect.width >= width && rect.height >= height {
                let leftover_horiz = (rect.width - width).abs();
                let leftover_vert = (rect.height - height).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = std::cmp::max(leftover_horiz, leftover_vert);
                if long_side_fit < best_long_side_fit
                    || (long_side_fit == best_long_side_fit && short_side_fit < best_short_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = best_long_side_fit;
                }
            }
            if rot && rect.width >= height && rect.height >= width {
                let leftover_horiz = (rect.width - height).abs();
                let leftover_vert = (rect.height - width).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);
                let long_side_fit = std::cmp::max(leftover_horiz, leftover_vert);
                if long_side_fit < best_long_side_fit
                    || (long_side_fit == best_long_side_fit && short_side_fit < best_short_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    best_short_side_fit = short_side_fit;
                    best_long_side_fit = best_long_side_fit;
                }
            }
        }

        (best_node, best_long_side_fit, best_short_side_fit)
    }
    fn find_position_for_new_node_best_area_fit(
        &self,
        rot: bool,
        width: i32,
        height: i32,
    ) -> (Rect, i32, i32) {
        let mut best_node = Rect::default();

        let mut best_area_fit = i32::max_value();
        let mut best_short_side_fit = i32::max_value();

        for rect in &self.free_rectangles {
            let area_fit = rect.width * rect.height - width * height;

            // Try to place the rectangle in upright (non-flipped) orientation
            if rect.width >= width && rect.height >= height {
                let leftover_horiz = (rect.width - width).abs();
                let leftover_vert = (rect.height - height).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);

                if area_fit < best_area_fit
                    || (area_fit == best_area_fit && short_side_fit < best_short_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    best_area_fit = area_fit;
                    best_short_side_fit = short_side_fit;
                }
            }
            if rot && rect.width >= height && rect.height >= width {
                let leftover_horiz = (rect.width - height).abs();
                let leftover_vert = (rect.height - width).abs();
                let short_side_fit = std::cmp::min(leftover_horiz, leftover_vert);

                if area_fit < best_area_fit
                    || (area_fit == best_area_fit && short_side_fit < best_short_side_fit)
                {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    best_area_fit = area_fit;
                    best_short_side_fit = short_side_fit;
                }
            }
        }

        (best_node, best_area_fit, best_short_side_fit)
    }
    fn find_position_for_new_node_contact_point(
        &self,
        rot: bool,
        width: i32,
        height: i32,
    ) -> (Rect, i32) {
        let mut best_node = Rect::default();

        let mut best_contact_score = -1;

        for rect in &self.free_rectangles {
            // Try to place the rectangle in upright (non-flipped) orientation
            if rect.width >= width && rect.height >= height {
                let score = self.contact_point_score_node(rect.x, rect.y, rect.width, rect.height);
                if score > best_contact_score {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    best_contact_score = score;
                }
            }
            if rot && rect.width >= height && rect.height >= width {
                let score = self.contact_point_score_node(rect.x, rect.y, rect.height, rect.width);
                if score > best_contact_score {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    best_contact_score = score;
                }
            }
        }

        (best_node, best_contact_score)
    }

    fn split_free_node(&mut self, free_node: &Rect, used_node: &Rect) -> bool {
        // Test if the rectangles even intersect.
        if used_node.x >= free_node.x + free_node.width
            || used_node.x + used_node.width <= free_node.x
            || used_node.y >= free_node.y + free_node.height
            || used_node.y + used_node.height <= free_node.y
        {
            return false;
        }

        if used_node.x < free_node.x + free_node.width
            && used_node.x + used_node.width > free_node.x
        {
            // New node at the top side of the used node
            if used_node.y > free_node.y && used_node.y < free_node.y + free_node.height {
                let mut new_node = free_node.clone();
                new_node.height = used_node.y - new_node.y;
                self.free_rectangles.push(new_node);
            }

            // New node at the bottom side of the used node
            if used_node.y + used_node.height < free_node.y + free_node.height {
                let mut new_node = free_node.clone();
                new_node.y = used_node.y + used_node.height;
                new_node.height = free_node.y + free_node.height - (used_node.y + used_node.height);
                self.free_rectangles.push(new_node);
            }
        }

        if used_node.y < free_node.y + free_node.height
            && used_node.y + used_node.height > free_node.y
        {
            // New node at the left side of the used node.
            if used_node.x > free_node.x && used_node.x < free_node.x + free_node.width {
                let mut new_node = free_node.clone();
                new_node.width = used_node.x - new_node.x;
                self.free_rectangles.push(new_node);
            }

            // New node at the right side of the used node
            if used_node.x + used_node.width < free_node.x + free_node.width {
                let mut new_node = free_node.clone();
                new_node.x = used_node.x + used_node.width;
                new_node.width = free_node.x + free_node.width - (used_node.x + used_node.width);
                self.free_rectangles.push(new_node);
            }
        }

        true
    }

    fn prune_free_list(&mut self) {
        let mut i = 0;
        while i < self.free_rectangles.len() {
            let mut j = i + 1;
            while j < self.free_rectangles.len() {
                let a = &self.free_rectangles[i];
                let b = &self.free_rectangles[j];
                if a.is_contained_in(b) {
                    self.free_rectangles.remove(i);
                    i -= 1;
                    break;
                }
                if b.is_contained_in(a) {
                    self.free_rectangles.remove(j);
                    j -= 1;
                }
                j += 1;
            }
            i += 1;
        }
    }
}

fn common_interval_length(i1start: i32, i1end: i32, i2start: i32, i2end: i32) -> i32 {
    if i1end < i2start || i2end < i1start {
        return 0;
    }
    return std::cmp::min(i1end, i2end) - std::cmp::max(i1start, i2start);
}
