use std::sync::{Arc, Weak};

use openxr::sys as xr;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use suinput::SuActionSet;
use thunderdome::Arena;

use crate::{
    str_from_bytes_until_nul,
    wrappers::layer_action::{self, LayerAction},
};

use super::{
    instance::{InnerInstance, InstanceWrapper},
    layer_action::SubActions,
};

pub struct LayerActionSet {
    pub instance: Weak<InstanceWrapper>,
    pub inner: Arc<InnerInstance>,
    pub su_action_set: SuActionSet,
}

pub fn all<'a>() -> RwLockReadGuard<'a, Arena<Arc<LayerActionSet>>> {
    unsafe { super::ACTION_SETS.get().unwrap().read() }
}

pub fn all_mut<'a>() -> RwLockWriteGuard<'a, Arena<Arc<LayerActionSet>>> {
    unsafe { super::ACTION_SETS.get().unwrap().write() }
}

impl LayerActionSet {
    pub fn xr_create_action(
        self: &Arc<Self>,
        create_info: &xr::ActionCreateInfo,
        handle_out: &mut xr::Action,
    ) -> Result<xr::Result, xr::Result> {
        let name = str_from_bytes_until_nul(&create_info.action_name[..])?;

        let index = layer_action::all_mut().insert(Arc::new(LayerAction {
            instance: self.instance.clone(),
            inner: self.inner.clone(),
            sub_actions: SubActions::new(&self.su_action_set, create_info, name),
        }));

        *handle_out = xr::Action::from_raw(index.to_bits());
        Ok(xr::Result::SUCCESS)
    }
}
