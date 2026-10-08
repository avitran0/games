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
    pub border_width: u32,
    pub corner_radius: u32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Color::Transparent,
            panel: Color::MineShaft,
            panel_border: Color::DoveGray,
            text: Color::White,
            muted_text: Color::Silver,
            widget: Color::Tundora,
            widget_focused: Color::Emperor,
            widget_pressed: Color::DodgerBlue,
            accent: Color::DodgerBlue,
            disabled: Color::Gray,
            row_height: 12,
            spacing: 3,
            padding: 5,
            border_width: 1,
            corner_radius: 2,
        }
    }
}
