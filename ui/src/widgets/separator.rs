use glam::ivec2;

use super::Widget;
use crate::{Response, Ui};

pub struct Separator;

impl Widget for Separator {
    fn show(self, ui: &mut Ui) -> Response {
        let position = ui.row_rect().position;
        let end = ivec2(
            position.x + ui.row_rect().size.x as i32,
            position.y + (ui.style.row_height / 2) as i32,
        );
        ui.draw_line(position, end, ui.style.panel_border);
        ui.advance(ui.style.row_height / 2 + ui.style.spacing);
        Response::default()
    }
}

impl Ui {
    pub fn separator(&mut self) -> Response {
        self.add(Separator)
    }
}
