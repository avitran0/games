pub mod button;
pub mod checkbox;
pub mod label;
pub mod progress_bar;
pub mod radio;
pub mod selectable;
pub mod slider;
pub mod tab_bar;

pub use button::Button;
pub use checkbox::Checkbox;
pub use label::Label;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use selectable::Selectable;
pub use slider::{Slider, SliderValue};
pub use tab_bar::TabBar;

use crate::{Response, Ui};

pub trait Widget {
    fn show(self, ui: &mut Ui) -> Response;
}
