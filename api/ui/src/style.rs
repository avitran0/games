use api::Color;

#[derive(Clone, Debug)]
pub struct Style {
    pub background: Color,
    pub panel: Color,
    pub panel_border: Color,
    pub text: Color,
    pub widget_text: Color,
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

impl Style {
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            panel: Color::Black,
            panel_border: Color::ShuttleGray,
            text: Color::Mischka,
            widget_text: Color::Mischka,
            focused_text: Color::Black,
            muted_text: Color::Chatelle,
            widget: Color::Black,
            widget_focused: Color::Mischka,
            widget_pressed: Color::Chatelle,
            accent: Color::Mischka,
            disabled: Color::ShuttleGray,
            row_height: 16,
            spacing: 3,
            padding: 6,
            outer_padding: 8,
            border_width: 1,
            corner_radius: 8,
            scrollbar_width: 3,
            scrollbar_track: Color::Black,
            scrollbar_thumb: Color::ShuttleGray,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Mischka,
            panel: Color::Chatelle,
            panel_border: Color::MountainMist,
            text: Color::Black,
            widget_text: Color::Mischka,
            focused_text: Color::Black,
            muted_text: Color::ShuttleGray,
            widget: Color::Black,
            widget_focused: Color::Mischka,
            widget_pressed: Color::Chatelle,
            accent: Color::Black,
            disabled: Color::ShuttleGray,
            row_height: 16,
            spacing: 3,
            padding: 6,
            outer_padding: 8,
            border_width: 1,
            corner_radius: 8,
            scrollbar_width: 3,
            scrollbar_track: Color::Chatelle,
            scrollbar_thumb: Color::ShuttleGray,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::dark()
    }
}
