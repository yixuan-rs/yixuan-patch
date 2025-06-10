use std::{thread, time::Duration};

use interceptor::Interceptor;
use modules::{
    censorship_patch::CensorshipPatch,
    crypto::{
        initialize_rsa_public_key, monitor_network_state, replace_sdk_public_key_string_literal,
    },
    hoyopass_patch::HoyopassPatch,
    network::Network,
    NapModuleManager,
};
use windows::{
    core::s,
    Win32::{
        Foundation::HINSTANCE,
        System::{Console, LibraryLoader::GetModuleHandleA, SystemServices::DLL_PROCESS_ATTACH},
    },
};

mod interceptor;
mod modules;
mod util;

unsafe fn thread_fn() {
    let _ = Console::AllocConsole();

    while GetModuleHandleA(s!("GameAssembly.dll")).is_err() {
        thread::sleep(Duration::from_millis(200));
    }

    thread::sleep(Duration::from_secs(5));
    util::disable_memory_protection();

    println!("yixuan-patch (2.1.1 BETA) is initializing");

    println!(
        "to work with yixuan-rs: https://git.xeondev.com/yixuan-rs/yixuan-rs/src/branch/2.1_beta"
    );

    println!("\nJoin us on Discord at https://discord.gg/reversedrooms\n\n\n");

    let mut module_manager = NapModuleManager::default();
    module_manager.add::<Network>();
    module_manager.add::<HoyopassPatch>();
    module_manager.add::<CensorshipPatch>();
    module_manager.init().expect("failed to initialize modules");

    initialize_rsa_public_key();
    replace_sdk_public_key_string_literal();

    let mut interceptor = Interceptor::default();
    monitor_network_state(&mut interceptor);

    thread::sleep(Duration::from_secs(u64::MAX));
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        thread::spawn(|| thread_fn());
    }

    true
}
