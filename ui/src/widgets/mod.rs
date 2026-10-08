pub mod button;
pub mod checkbox;
pub mod label;
pub mod progress_bar;
pub mod radio;
pub mod selectable;
pub mod separator;
pub mod slider;

pub use button::Button;
pub use checkbox::Checkbox;
pub use label::Label;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use selectable::Selectable;
pub use separator::Separator;
pub use slider::Slider;

use crate::{Response, Ui};

pub trait Widget {
    fn show(self, ui: &mut Ui) -> Response;
}
