use ilhook::x64::Registers;
use std::sync::atomic::{AtomicU32, Ordering};

use super::{ModuleInitError, NapModule, NapModuleContext};

const SET_DITHER_CONFIG: usize = 0x8CA93D0;
const DITHER_CONFIG_AVATAR_USING_DITHER_ALPHA: usize = 0x49;

const ON_ENTER_SCENE_SC_NOTIFY: usize = 0x86E6F80;
const ENTER_SCENE_SC_NOTIFY_SCENE_DATA: usize = 0x10;
const SCENE_DATA_SCENE_TYPE: usize = 0x74;
const SCENE_TYPE_HALL: u32 = 1;

static LAST_ENTER_SCENE_TYPE: AtomicU32 = AtomicU32::new(0);

pub struct CensorshipPatch;

impl NapModule for NapModuleContext<CensorshipPatch> {
    unsafe fn init(&mut self) -> Result<(), ModuleInitError> {
        self.interceptor.attach(
            self.base.wrapping_add(SET_DITHER_CONFIG),
            CensorshipPatch::on_set_dither_config,
        )?;

        self.interceptor.attach(
            self.base.wrapping_add(ON_ENTER_SCENE_SC_NOTIFY),
            CensorshipPatch::on_enter_scene_sc_notify,
        )?;

        Ok(())
    }
}

impl CensorshipPatch {
    pub unsafe extern "win64" fn on_set_dither_config(reg: *mut Registers, _: usize) {
        if LAST_ENTER_SCENE_TYPE.load(Ordering::SeqCst) == SCENE_TYPE_HALL {
            if (*reg).rdx != 0 {
                println!("SetDitherConfig: disabling dither alpha");
                *(((*reg).rdx as *mut u8).wrapping_add(DITHER_CONFIG_AVATAR_USING_DITHER_ALPHA)) = 0;
            }
        } else {
            println!("SetDitherConfig: not in hall, ignoring");
        }
    }

    pub unsafe extern "win64" fn on_enter_scene_sc_notify(reg: *mut Registers, _: usize) {
        let scene_data = *((*reg).rdx as *const u8)
            .wrapping_add(ENTER_SCENE_SC_NOTIFY_SCENE_DATA)
            .cast::<usize>();

        let scene_type = *(scene_data as *const u8)
            .wrapping_add(SCENE_DATA_SCENE_TYPE)
            .cast::<u32>();

        println!("EnterSceneScNotify scene_type: {scene_type}");

        LAST_ENTER_SCENE_TYPE.store(scene_type, Ordering::SeqCst);
    }
}
