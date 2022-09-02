use std::sync::{Arc, Weak};

use openxr::sys as xr;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use suinput::{
    action_type::{Axis1d, Axis2d},
    SuAction, SuActionSet,
};
use thunderdome::{Arena, Index};

use super::instance::{InnerInstance, InstanceWrapper};

pub struct LayerAction {
    pub instance: Weak<InstanceWrapper>,
    pub inner: Arc<InnerInstance>,
    pub sub_actions: SubActions,
}

pub fn all<'a>() -> RwLockReadGuard<'a, Arena<Arc<LayerAction>>> {
    unsafe { super::ACTIONS.get().unwrap().read() }
}

pub fn all_mut<'a>() -> RwLockWriteGuard<'a, Arena<Arc<LayerAction>>> {
    unsafe { super::ACTIONS.get().unwrap().write() }
}

pub fn get(arena: &Arena<Arc<LayerAction>>, handle: xr::Action) -> openxr::Result<&Arc<LayerAction>> {
    let index = match Index::from_bits(handle.into_raw()) {
        Some(index) => index,
        None => return Err(xr::Result::ERROR_HANDLE_INVALID),
    };
    arena.get(index).ok_or(xr::Result::ERROR_HANDLE_INVALID)
}

pub enum SubActions {
    None(SingletonAction),
    Some(ManySubActions),
}

impl SubActions {
    pub fn new(action_set: &SuActionSet, create_info: &xr::ActionCreateInfo, name: &str) -> Self {
        if create_info.count_subaction_paths == 0 {
            return Self::None(match create_info.action_type {
                xr::ActionType::BOOLEAN_INPUT => {
                    SingletonAction::Boolean(action_set.create_action(name, Default::default()))
                }
                xr::ActionType::FLOAT_INPUT => {
                    SingletonAction::Float(action_set.create_action(name, Default::default()))
                }
                xr::ActionType::VECTOR2F_INPUT => {
                    SingletonAction::Vector2f(action_set.create_action(name, Default::default()))
                }
                xr::ActionType::POSE_INPUT => SingletonAction::Pose(()),
                xr::ActionType::VIBRATION_OUTPUT => SingletonAction::Vibration(()),
                _ => todo!("TODO handle unknown action type"),
            });
        }

        let sub_action_paths = unsafe {
            std::slice::from_raw_parts(
                create_info.subaction_paths,
                create_info.count_subaction_paths as usize,
            )
        }
        .iter();

        Self::Some(match create_info.action_type {
            xr::ActionType::BOOLEAN_INPUT => ManySubActions::Boolean(
                sub_action_paths
                    .map(|path| (*path, action_set.create_action(name, Default::default())))
                    .collect(),
            ),
            xr::ActionType::FLOAT_INPUT => ManySubActions::Float(
                sub_action_paths
                    .map(|path| (*path, action_set.create_action(name, Default::default())))
                    .collect(),
            ),
            xr::ActionType::VECTOR2F_INPUT => ManySubActions::Vector2f(
                sub_action_paths
                    .map(|path| (*path, action_set.create_action(name, Default::default())))
                    .collect(),
            ),
            xr::ActionType::POSE_INPUT => ManySubActions::Pose(()),
            xr::ActionType::VIBRATION_OUTPUT => ManySubActions::Vibration(()),
            _ => todo!("TODO handle unknown action type"),
        })
    }
}

pub enum SingletonAction {
    Boolean(SuAction<bool>),
    Float(SuAction<Axis1d>),
    Vector2f(SuAction<Axis2d>),
    Pose(()),
    Vibration(()),
}

pub enum ManySubActions {
    Boolean(Vec<(xr::Path, SuAction<bool>)>),
    Float(Vec<(xr::Path, SuAction<Axis1d>)>),
    Vector2f(Vec<(xr::Path, SuAction<Axis2d>)>),
    Pose(()),
    Vibration(()),
}

#[derive(Debug, Default)]
pub struct BooleanState {
    enabled: bool,
    changed: bool,
    state: bool,
}

#[derive(Debug, Default)]
pub struct FloatState {
    enabled: bool,
    changed: bool,
    state: f32,
}

#[derive(Debug, Default)]
pub struct Vector2fState {
    enabled: bool,
    changed: bool,
    state: xr::Vector2f,
}

#[derive(Debug, Default)]
pub struct PoseState {
    enabled: bool,
}
