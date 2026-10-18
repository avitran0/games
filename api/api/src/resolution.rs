use glam::{UVec2, uvec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::THREE_TWO
    }
}

impl Resolution {
    pub const FOUR_THREE: Self = Self::new(224, 168);
    pub const THREE_TWO: Self = Self::new(240, 160);
    pub const SIXTEEN_TEN: Self = Self::new(256, 160);
    pub const SIXTEEN_NINE: Self = Self::new(256, 144);

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn for_window_size(width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            return Self::THREE_TWO;
        }

        let aspect = width as f64 / height as f64;
        [
            Self::FOUR_THREE,
            Self::THREE_TWO,
            Self::SIXTEEN_TEN,
            Self::SIXTEEN_NINE,
        ]
        .into_iter()
        .min_by(|a, b| {
            let a_aspect = a.width as f64 / a.height as f64;
            let b_aspect = b.width as f64 / b.height as f64;
            (aspect - a_aspect)
                .abs()
                .total_cmp(&(aspect - b_aspect).abs())
        })
        .unwrap_or(Self::THREE_TWO)
    }
}

impl From<Resolution> for UVec2 {
    fn from(val: Resolution) -> Self {
        uvec2(val.width, val.height)
    }
}

#[cfg(test)]
mod tests {
    use super::Resolution;

    #[test]
    fn selects_nearest_supported_aspect_ratio() {
        assert_eq!(
            Resolution::for_window_size(640, 480),
            Resolution::FOUR_THREE
        );
        assert_eq!(Resolution::for_window_size(720, 480), Resolution::THREE_TWO);
        assert_eq!(
            Resolution::for_window_size(1280, 800),
            Resolution::SIXTEEN_TEN
        );
        assert_eq!(
            Resolution::for_window_size(1920, 1080),
            Resolution::SIXTEEN_NINE
        );
    }

    #[test]
    fn three_two_resolution_scales_exactly_to_720_by_480() {
        assert_eq!(Resolution::THREE_TWO, Resolution::new(240, 160));
        assert_eq!(Resolution::THREE_TWO.width * 3, 720);
        assert_eq!(Resolution::THREE_TWO.height * 3, 480);
    }

    #[test]
    fn zero_sized_window_uses_default_resolution() {
        assert_eq!(Resolution::for_window_size(0, 0), Resolution::THREE_TWO);
    }
}
