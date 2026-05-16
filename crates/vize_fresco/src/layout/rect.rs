//! Rectangle and area calculations.

use serde::{Deserialize, Serialize};

/// A rectangle representing an area in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rect {
    /// X coordinate (column)
    pub x: u16,
    /// Y coordinate (row)
    pub y: u16,
    /// Width in columns
    pub width: u16,
    /// Height in rows
    pub height: u16,
}

impl Rect {
    /// Create a new rectangle.
    #[inline]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle at origin with given dimensions.
    #[inline]
    pub const fn sized(width: u16, height: u16) -> Self {
        Self::new(0, 0, width, height)
    }

    /// Create an empty rectangle.
    #[inline]
    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Check if the rectangle is empty (zero area).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get the area (width * height).
    #[inline]
    pub const fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Get the left edge x coordinate.
    #[inline]
    pub const fn left(&self) -> u16 {
        self.x
    }

    /// Get the right edge x coordinate (exclusive).
    #[inline]
    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Get the top edge y coordinate.
    #[inline]
    pub const fn top(&self) -> u16 {
        self.y
    }

    /// Get the bottom edge y coordinate (exclusive).
    #[inline]
    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Check if a point is inside the rectangle.
    #[inline]
    pub const fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && x < self.x.saturating_add(self.width)
            && y >= self.y
            && y < self.y.saturating_add(self.height)
    }

    /// Compute the intersection with another rectangle.
    pub fn intersection(&self, other: &Rect) -> Rect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x1 < x2 && y1 < y2 {
            Rect::new(x1, y1, x2 - x1, y2 - y1)
        } else {
            Rect::empty()
        }
    }

    /// Compute the union (bounding box) with another rectangle.
    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());

        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Shrink the rectangle by a margin on all sides.
    pub fn inner(&self, margin: u16) -> Rect {
        let margin2 = margin.saturating_mul(2);
        if self.width < margin2 || self.height < margin2 {
            return Rect::empty();
        }

        Rect::new(
            self.x.saturating_add(margin),
            self.y.saturating_add(margin),
            self.width - margin2,
            self.height - margin2,
        )
    }

    /// Shrink the rectangle by different margins.
    pub fn inner_asymmetric(&self, top: u16, right: u16, bottom: u16, left: u16) -> Rect {
        let new_width = self.width.saturating_sub(left).saturating_sub(right);
        let new_height = self.height.saturating_sub(top).saturating_sub(bottom);

        if new_width == 0 || new_height == 0 {
            return Rect::empty();
        }

        Rect::new(
            self.x.saturating_add(left),
            self.y.saturating_add(top),
            new_width,
            new_height,
        )
    }

    /// Expand the rectangle by a margin on all sides.
    pub fn outer(&self, margin: u16) -> Rect {
        Rect::new(
            self.x.saturating_sub(margin),
            self.y.saturating_sub(margin),
            self.width.saturating_add(margin.saturating_mul(2)),
            self.height.saturating_add(margin.saturating_mul(2)),
        )
    }

    /// Offset the rectangle by (dx, dy).
    pub fn offset(&self, dx: i16, dy: i16) -> Rect {
        Rect::new(
            (self.x as i32 + dx as i32).max(0) as u16,
            (self.y as i32 + dy as i32).max(0) as u16,
            self.width,
            self.height,
        )
    }

    /// Clamp this rectangle to fit within another.
    pub fn clamp(&self, bounds: &Rect) -> Rect {
        self.intersection(bounds)
    }
}

impl From<(u16, u16, u16, u16)> for Rect {
    fn from((x, y, width, height): (u16, u16, u16, u16)) -> Self {
        Self::new(x, y, width, height)
    }
}

impl From<Rect> for (u16, u16, u16, u16) {
    fn from(rect: Rect) -> Self {
        (rect.x, rect.y, rect.width, rect.height)
    }
}

#[cfg(test)]
mod tests {
    use super::Rect;

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10, 20, 30, 40);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 30);
        assert_eq!(rect.height, 40);
    }

    #[test]
    fn test_rect_edges() {
        let rect = Rect::new(10, 20, 30, 40);
        assert_eq!(rect.left(), 10);
        assert_eq!(rect.right(), 40);
        assert_eq!(rect.top(), 20);
        assert_eq!(rect.bottom(), 60);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 20, 20);
        assert!(rect.contains(10, 10));
        assert!(rect.contains(29, 29));
        assert!(!rect.contains(9, 10));
        assert!(!rect.contains(30, 30));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        let inter = a.intersection(&b);
        assert_eq!(inter, Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn test_rect_no_intersection() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(20, 20, 10, 10);
        let inter = a.intersection(&b);
        assert!(inter.is_empty());
    }

    #[test]
    fn test_rect_inner() {
        let rect = Rect::new(0, 0, 20, 20);
        let inner = rect.inner(2);
        assert_eq!(inner, Rect::new(2, 2, 16, 16));
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0, 0, 10, 20);
        assert_eq!(rect.area(), 200);
    }
}
