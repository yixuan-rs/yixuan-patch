use std::ffi::CString;

use ilhook::x64::Registers;

use crate::{
    interceptor::Interceptor,
    util::{import, GAME_ASSEMBLY_BASE},
};

import!(rsa_create() -> usize = 0x1B582F20);
import!(rsa_from_xml_string(instance: usize, xml_string: usize) -> usize = 0x1B583160);
import!(il2cpp_string_new(cstr: *const u8) -> usize = 0x1158AA0);

pub unsafe fn initialize_rsa_public_key() {
    const SERVER_PUBLIC_KEY: &str = include_str!("../../server_public_key.xml");
    let rsa_public_key_backdoor_field =
        ((*(GAME_ASSEMBLY_BASE.wrapping_add(0x554E500) as *const usize)) + 252856) as *mut usize;

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

    *(GAME_ASSEMBLY_BASE.wrapping_add(0x5950C50) as *mut usize) = il2cpp_string_new(
        CString::new(SDK_PUBLIC_KEY)
            .unwrap()
            .to_bytes_with_nul()
            .as_ptr(),
    ) as usize;

    *(GAME_ASSEMBLY_BASE.wrapping_add(0x5974060) as *mut usize) = il2cpp_string_new(
        [27818, 40348, 47410, 27936, 51394, 33172, 51987, 8709, 44748,
        23705, 45753, 21092, 57054, 52661, 369, 62630, 11725, 7496, 36921, 28271,
        34880, 52645, 31515, 18214, 3108, 2077, 13490, 25459, 58590, 47504, 15163,
        8951, 44748, 23705, 45753, 29284, 57054, 52661]
        .into_iter()
        .enumerate()
        .flat_map(|(i, v)| {
            let b = (((i + ((i >> 31) >> 29)) & 0xF8).wrapping_sub(i)) as i16;
            (((v << ((b + 11) & 0xF)) | (v >> ((-11 - b) & 0xF))) & 0xFFFF_u16)
                .to_be_bytes()
                .into_iter()
        })
        .chain([0])
        .collect::<Vec<_>>()
        .as_ptr(),
    ) as usize;
}

pub unsafe fn monitor_network_state(interceptor: &mut Interceptor) {
    interceptor
        .attach(
            GAME_ASSEMBLY_BASE.wrapping_add(0xDE96720),
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
