use api::{Button, ScreenAction};
use ui::{Alignment, Rect, Ui};

use crate::{
    core::{block::BlockCategory, block_kind::BlockKind},
    state::State,
};

const PANEL: Rect = Rect::new(8, 8, 224, 144);

pub struct BlockSelectionScreen {
    ui: Ui,
    category: BlockCategory,
}

impl BlockSelectionScreen {
    pub fn new(ctx: &mut api::ScreenContext<'_, State>) -> Self {
        let mut ui = Ui::new();
        ui.set_alignment(Alignment::Left);
        Self {
            ui,
            category: ctx
                .state
                .picked_block
                .map_or(BlockCategory::Logistics, BlockKind::category),
        }
    }
}

impl api::Screen<State> for BlockSelectionScreen {
    fn update(&mut self, ctx: &mut api::ScreenContext<'_, State>) -> ScreenAction<State> {
        if ctx.input.just_pressed(Button::B) {
            return ScreenAction::Pop;
        }

        self.ui.begin_frame(ctx);
        self.ui.set_bounds(PANEL);

        let mut picked_block = ctx.state.picked_block;
        let mut picked_now = false;
        let title = format!("Build · {}", self.category.label());
        self.ui.panel(PANEL, &title, |ui| {
            ui.tab_bar(&mut self.category, |ui, category| {
                for block in category.blocks() {
                    let size = block.size().cells();
                    let label = format!("{}  {}x{}", block.label(), size, size);
                    picked_now |= ui.radio(label, &mut picked_block, Some(block)).clicked;
                }
            });
        });
        ctx.state.picked_block = picked_block;

        if picked_now {
            return ScreenAction::Pop;
        }

        ScreenAction::None
    }

    fn draw(&mut self, _state: &State, frame: &mut api::Frame) {
        self.ui.paint(frame);
    }

    fn is_modal(&self) -> bool {
        true
    }
}
