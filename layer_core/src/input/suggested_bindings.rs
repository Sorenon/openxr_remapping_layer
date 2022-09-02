use std::{collections::HashMap, sync::Arc};
use openxr as xr;
use thunderdome::Index;

use crate::wrappers::layer_action_set::LayerActionSet;

pub struct InteractionProfileSuggestedBindings {
    pub action_sets: HashMap<Arc<LayerActionSet>, SuggestedBindings>,
}

pub struct SuggestedBindings {
    pub bindings: Vec<SuggestedBinding>,
}

pub enum SuggestedBinding {
    SimpleBinding {
        action: Index,
        binding: xr::Path,
    },
    AnalogThreshold {
        action: Index,
        binding: xr::Path,
        on_threshold: f32,
        off_threshold: f32,
        on_haptic: (),
        off_haptic: (),
    },
    DPadBinding {
        binding: xr::Path,
        // action_set: Index,
        force_threshold: f32,
        force_threshold_released: f32,
        center_region: f32,
        wedge_angle: f32,
        is_sticky: bool,
        on_haptic: (),
        off_haptic: (),
    }
}

impl SuggestedBinding {
    // pub fn is_valid(&self) -> bool {

    // }
}