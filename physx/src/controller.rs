#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use crate::px_type::*;
use crate::rigid_dynamic::RigidDynamic;

use crate::transform::gl_to_px_v3;
use glam::Vec3;
use physx_macros::*;
use physx_sys::PxControllerFilterCallback;
use physx_sys::{
    phys_PxCreateControllerManager, PxCapsuleControllerDesc, PxCapsuleControllerDesc_delete,
    PxCapsuleControllerDesc_isValid, PxCapsuleControllerDesc_new_alloc, PxController,
    PxControllerDesc, PxControllerFilters, PxControllerFilters_new, PxControllerManager,
    PxControllerManager_createController_mut, PxController_getActor, PxController_getPosition,
    PxController_move_mut, PxController_release_mut, PxController_setPosition_mut, PxExtendedVec3,
    PxMaterial, PxQueryFilterCallback, PxScene,
};
use std::ptr::null;

#[physx_type]
impl ControllerDesc {}

#[physx_type]
impl ControllerManager {
    pub fn new(scene: *mut PxScene) -> Self {
        let ptr = unsafe { phys_PxCreateControllerManager(scene, false) };
        ControllerManager { ptr }
    }

    pub fn create_controller(&self, desc: &mut ControllerDesc) -> Controller {
        let controller =
            unsafe { PxControllerManager_createController_mut(self.ptr, desc.get_raw_mut()) };
        Controller::new(controller)
    }
}

#[physx_type(inherit = "ControllerDesc")]
impl CapsuleControllerDesc {
    pub fn new(
        height: f32,
        radius: f32,
        step_offset: f32,
        material: *mut PxMaterial,
    ) -> Result<Self, String> {
        unsafe {
            let c = PxCapsuleControllerDesc_new_alloc();
            (*c).height = height;
            (*c).radius = radius;
            (*c).stepOffset = step_offset;
            (*c).material = material;

            if PxCapsuleControllerDesc_isValid(c) {
                Ok(CapsuleControllerDesc { ptr: c })
            } else {
                Err(format!(
                    "Controller description is invalid. height={}, radius={}, step_offset={}",
                    height, radius, step_offset
                ))
            }
        }
    }

    pub fn release(self: Self) {
        unsafe { PxCapsuleControllerDesc_delete(self.ptr) };
    }
}

pub struct Controllable {
    controller: Controller,
    filters: PxControllerFilters,
}

#[physx_type]
impl Controller {
    pub fn new(controller: *mut PxController) -> Self {
        Self { ptr: controller }
    }

    pub fn set_position(&mut self, position: Vec3) {
        unsafe {
            PxController_setPosition_mut(self.get_raw_mut(), &to_extended(&position));
        }
    }

    pub fn get_position(&self) -> Vec3 {
        unsafe { from_extended(*PxController_getPosition(self.get_raw())) }
    }

    pub fn get_actor(&self) -> RigidDynamic {
        unsafe { RigidDynamic::new(PxController_getActor(self.ptr)) }
    }

    pub fn release(&mut self) {
        unsafe {
            PxController_release_mut(self.get_raw_mut());
        }
    }
}

impl Controllable {
    pub fn new(controller: Controller) -> Self {
        let filters = unsafe {
            PxControllerFilters_new(
                null(),
                null::<PxQueryFilterCallback>() as *mut PxQueryFilterCallback,
                null::<PxControllerFilterCallback>() as *mut PxControllerFilterCallback,
            )
        };
        Controllable {
            controller,
            filters,
        }
    }

    pub fn move_by(&mut self, disp: Vec3, min_distance: f32, elapsed_time: f32) {
        unsafe {
            PxController_move_mut(
                self.controller.get_raw_mut(),
                &gl_to_px_v3(disp),
                min_distance,
                elapsed_time,
                &self.filters,
                null(),
            );
        }
    }
}

impl Deref for Controllable {
    type Target = Controller;
    fn deref(&self) -> &Controller {
        &self.controller
    }
}

impl DerefMut for Controllable {
    fn deref_mut(&mut self) -> &mut Controller {
        &mut self.controller
    }
}

fn to_extended(vec: &Vec3) -> PxExtendedVec3 {
    PxExtendedVec3 {
        x: vec.x() as f64,
        y: vec.y() as f64,
        z: vec.z() as f64,
    }
}

fn from_extended(vec: PxExtendedVec3) -> Vec3 {
    Vec3::new(vec.x as f32, vec.y as f32, vec.z as f32)
}
