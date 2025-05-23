use std::ffi::CString;

use ilhook::x64::Registers;

use crate::{
    interceptor::Interceptor,
    util::{import, GAME_ASSEMBLY_BASE},
};

import!(rsa_create() -> usize = 0x1B56B2E0);
import!(rsa_from_xml_string(instance: usize, xml_string: usize) -> usize = 0x1B56B520);
import!(il2cpp_string_new(cstr: *const u8) -> usize = 0x115F1B0);

pub unsafe fn initialize_rsa_public_key() {
    const SERVER_PUBLIC_KEY: &str = include_str!("../../server_public_key.xml");
    let rsa_public_key_backdoor_field =
        ((*(GAME_ASSEMBLY_BASE.wrapping_add(0x5552100) as *const usize)) + 252792) as *mut usize;

    let rsa = rsa_create();
    rsa_from_xml_string(
        rsa,
        il2cpp_string_new(
            CString::new(SERVER_PUBLIC_KEY)
                .unwrap()
                .to_bytes_with_nul()
                .as_ptr(),
        ),
    );

    *rsa_public_key_backdoor_field = rsa;
}

pub unsafe fn replace_sdk_public_key_string_literal() {
    const SDK_PUBLIC_KEY: &str = include_str!("../../sdk_public_key.xml");

    *(GAME_ASSEMBLY_BASE.wrapping_add(0x53D49C0) as *mut usize) = il2cpp_string_new(
        CString::new(SDK_PUBLIC_KEY)
            .unwrap()
            .to_bytes_with_nul()
            .as_ptr(),
    ) as usize;
}

pub unsafe fn monitor_network_state(interceptor: &mut Interceptor) {
    interceptor
        .attach(
            GAME_ASSEMBLY_BASE.wrapping_add(0xD8AAEC0),
            on_network_state_change,
        )
        .unwrap();
}

unsafe extern "win64" fn on_network_state_change(reg: *mut Registers, _: usize) {
    let net_state = NetworkState::from((*reg).rcx);
    println!("network state change: {net_state:?}");

    if net_state == NetworkState::PlayerLoginCsReq {
        // public key rsa gets reset to null after successful PlayerGetTokenScRsp
        initialize_rsa_public_key();
    }
}

#[repr(u64)]
#[derive(num_enum::FromPrimitive, Debug, Default, PartialEq)]
pub enum NetworkState {
    CloudCmdLine = 1021,
    CloudDispatch = 1020,
    StartBasicsReq = 17,
    LoadShaderEnd = 9,
    PlayerLoginCsReq = 15,
    EndBasicsReq = 18,
    LoadResourcesEnd = 10,
    GlobalDispatch = 1,
    ConnectGameServer = 12,
    ChooseServer = 2,
    DoFileVerifyEnd = 7,
    PlayerLoginScRsp = 16,
    DispatchResult = 4,
    PlayerGetTokenScRsp = 14,
    DownloadResourcesEnd = 6,
    AccountLogin = 3,
    LoadAssetEnd = 8,
    StartEnterGameWorld = 11,
    #[default]
    None = 0,
    EnterWorldScRsp = 19,
    PlayerGetTokenCsReq = 13,
    StartDownLoad = 5,
    DoFileVerifyFailed = 1022,
    CleanExpireEnd = 1023,
}
