/// Fixed height per row (px) for virtual scrolling.
pub const ITEM_HEIGHT_PX: f64 = 22.0;

/// Extra rows rendered above/below the visible area.
const OVERSCAN: usize = 5;

/// Calculate the visible index range `[start, end)` for virtual scrolling.
///
/// Clamps scroll_top to valid range so stale values (e.g. after results change)
/// don't cause out-of-bounds rendering.
pub fn visible_range(scroll_top: f64, container_height: f64, total_items: usize) -> (usize, usize) {
    if total_items == 0 {
        return (0, 0);
    }
    let max_scroll = (total_items as f64 * ITEM_HEIGHT_PX - container_height).max(0.0);
    let clamped_scroll = scroll_top.clamp(0.0, max_scroll);

    let start = ((clamped_scroll / ITEM_HEIGHT_PX) as usize).saturating_sub(OVERSCAN);
    let visible_count = (container_height / ITEM_HEIGHT_PX).ceil() as usize;
    let end = (start + visible_count + OVERSCAN * 2).min(total_items);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_items() {
        assert_eq!(visible_range(0.0, 500.0, 0), (0, 0));
    }

    #[test]
    fn start_of_list() {
        let (start, end) = visible_range(0.0, 220.0, 100);
        assert_eq!(start, 0);
        // visible_count = ceil(220/22) = 10, end = min(0+10+10, 100) = 20
        assert_eq!(end, 20);
    }

    #[test]
    fn middle_of_list() {
        // scroll_top = 440 → first visible = 20
        let (start, end) = visible_range(440.0, 220.0, 100);
        // start = 20 - 5 = 15
        assert_eq!(start, 15);
        // end = min(15 + 10 + 10, 100) = 35
        assert_eq!(end, 35);
    }

    #[test]
    fn clamps_stale_scroll() {
        // scroll_top way beyond content
        let (start, end) = visible_range(99999.0, 220.0, 10);
        // max_scroll = (10*22 - 220).max(0) = 0
        // clamped = 0 → start = 0, end = min(0+10+10, 10) = 10
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn few_items_all_visible() {
        let (start, end) = visible_range(0.0, 800.0, 5);
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }
}
