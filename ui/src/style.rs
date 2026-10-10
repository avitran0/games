use api::Color;

#[derive(Clone, Debug)]
pub struct Style {
    pub background: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub text: Color,
    pub focused_text: Color,
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
            background: Color::Black,
            panel: Color::Black,
            panel_border: Color::Black,
            text: Color::White,
            focused_text: Color::Black,
            muted_text: Color::FrenchGray,
            widget: Color::Black,
            widget_focused: Color::White,
            widget_pressed: Color::FrenchGray,
            accent: Color::White,
            disabled: Color::ShuttleGray,
            row_height: 16,
            spacing: 3,
            padding: 8,
            outer_padding: 8,
            border_width: 0,
            corner_radius: 50,
            scrollbar_width: 3,
            scrollbar_track: Color::Bunker,
            scrollbar_thumb: Color::ShuttleGray,
        }
    }
}
