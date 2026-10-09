use api::Color;

#[derive(Clone, Debug)]
pub struct Style {
    pub background: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub text: Color,
    pub muted_text: Color,
    pub widget: Color,
    pub widget_focused: Color,
    pub widget_pressed: Color,
    pub accent: Color,
    pub disabled: Color,
    pub row_height: u32,
    pub spacing: u32,
    pub padding: u32,
    pub outer_padding: u32,
    pub border_width: u32,
    pub corner_radius: u32,
    pub scrollbar_width: u32,
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Color::Bunker,
            panel: Color::EbonyClay,
            panel_border: Color::EbonyClay2,
            text: Color::White,
            muted_text: Color::Raven,
            widget: Color::EbonyClay,
            widget_focused: Color::EbonyClay2,
            widget_pressed: Color::BrightGray,
            accent: Color::Malibu,
            disabled: Color::Raven,
            row_height: 15,
            spacing: 3,
            padding: 4,
            outer_padding: 4,
            border_width: 1,
            corner_radius: 2,
            scrollbar_width: 3,
            scrollbar_track: Color::EbonyClay2,
            scrollbar_thumb: Color::Raven,
        }
    }
}
