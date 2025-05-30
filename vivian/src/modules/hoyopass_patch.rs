use ilhook::x64::Registers;

use crate::util::GAME_ASSEMBLY_BASE;

use super::{ModuleInitError, NapModule, NapModuleContext};

const ON_COMBO_INIT_SUCCESS: usize = 0x1ACEFC40;
const STATICS: usize = 0x55594B8;
const STATIC_ID: usize = 34496;
const FIELD_OFFSET: usize = 0x44;

pub struct HoyopassPatch;

impl NapModule for NapModuleContext<HoyopassPatch> {
    unsafe fn init(&mut self) -> Result<(), ModuleInitError> {
        self.interceptor.attach(
            self.base.wrapping_add(ON_COMBO_INIT_SUCCESS),
            HoyopassPatch::on_combo_init_success,
        )?;

        Ok(())
    }
}

impl HoyopassPatch {
    pub unsafe extern "win64" fn on_combo_init_success(_: *mut Registers, _: usize) {
        let statics = *(GAME_ASSEMBLY_BASE.wrapping_add(STATICS) as *mut usize);
        let config_manager = *(statics.wrapping_add(STATIC_ID) as *mut usize);
        *(config_manager.wrapping_add(FIELD_OFFSET) as *mut bool) = false;
        println!("HoYoPassPatch - OnComboInitSuccess()");
    }
}
