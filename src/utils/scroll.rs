pub fn calculate_start_index(scroll_y: f64, line_height: f64, top_buffer: usize) -> usize {
    let raw_index = (scroll_y / line_height).floor() as usize;
    raw_index.saturating_sub(top_buffer)
}

pub fn calculate_window_size(viewport_height: f64, line_height: f64, buffer_size: usize) -> usize {
    let visible_lines = (viewport_height / line_height).ceil() as usize;
    visible_lines + buffer_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_math() {
        let line_height = 20.0;
        let top_buffer = 5;

        // Scrolled 0px -> index 0
        assert_eq!(calculate_start_index(0.0, line_height, top_buffer), 0);

        // Scrolled 100px (5 lines) -> index 0 (5 - 5 = 0)
        assert_eq!(calculate_start_index(100.0, line_height, top_buffer), 0);

        // Scrolled 120px (6 lines) -> index 1 (6 - 5 = 1)
        assert_eq!(calculate_start_index(120.0, line_height, top_buffer), 1);

        // Viewport 600px (30 lines), buffer 15 -> 45
        assert_eq!(calculate_window_size(600.0, line_height, 15), 45);

        // Viewport 610px (30.5 lines -> 31 lines), buffer 15 -> 46
        assert_eq!(calculate_window_size(610.0, line_height, 15), 46);
    }
}
