//! Keeps the main window reachable. A window position restored by the window-state plugin can
//! lie outside every connected monitor (external display unplugged, resolution change), which
//! makes the window "disappear" as soon as the user touches it.

/// Rectangle in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// Minimum visible part of the window (in each direction) that counts as "on screen".
const MIN_VISIBLE: i64 = 120;

fn overlap(a: Rect, b: Rect) -> (i64, i64) {
    let ox = (a.x as i64 + a.w as i64).min(b.x as i64 + b.w as i64) - (a.x as i64).max(b.x as i64);
    let oy = (a.y as i64 + a.h as i64).min(b.y as i64 + b.h as i64) - (a.y as i64).max(b.y as i64);
    (ox, oy)
}

/// Returns the position the window should be moved to, or `None` when it is already usable.
/// `monitors` are the work areas of all connected monitors, the first one is the fallback.
pub fn fix_position(window: Rect, monitors: &[Rect]) -> Option<(i32, i32)> {
    let fallback = *monitors.first()?;
    let visible = monitors.iter().any(|m| {
        let (ox, oy) = overlap(window, *m);
        ox >= MIN_VISIBLE.min(window.w as i64) && oy >= MIN_VISIBLE.min(window.h as i64)
    });
    if visible {
        return None;
    }
    // Center on the fallback monitor; a window larger than the monitor is pinned to its corner.
    let x = fallback.x as i64 + ((fallback.w as i64 - window.w as i64) / 2).max(0);
    let y = fallback.y as i64 + ((fallback.h as i64 - window.h as i64) / 2).max(0);
    Some((x as i32, y as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN: Rect = Rect { x: 0, y: 0, w: 1920, h: 1080 };
    const SIDE: Rect = Rect { x: 1920, y: 0, w: 1920, h: 1080 };

    #[test]
    fn keeps_window_inside_monitor() {
        let w = Rect { x: 100, y: 100, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[MAIN]), None);
    }

    #[test]
    fn keeps_window_on_second_monitor() {
        let w = Rect { x: 2200, y: 50, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[MAIN, SIDE]), None);
    }

    #[test]
    fn recovers_window_of_unplugged_monitor() {
        let w = Rect { x: 2200, y: 50, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[MAIN]), Some((460, 190)));
    }

    #[test]
    fn recovers_window_with_only_a_sliver_visible() {
        let w = Rect { x: 1900, y: 100, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[MAIN]), Some((460, 190)));
    }

    #[test]
    fn recovers_window_far_above_the_screen() {
        let w = Rect { x: 100, y: -5000, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[MAIN]), Some((460, 190)));
    }

    #[test]
    fn window_larger_than_monitor_goes_to_corner() {
        let w = Rect { x: 5000, y: 5000, w: 3000, h: 2000 };
        assert_eq!(fix_position(w, &[MAIN]), Some((0, 0)));
    }

    #[test]
    fn no_monitors_means_no_change() {
        let w = Rect { x: 5000, y: 5000, w: 1000, h: 700 };
        assert_eq!(fix_position(w, &[]), None);
    }
}
