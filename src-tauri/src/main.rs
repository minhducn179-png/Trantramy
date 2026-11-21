// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Prevent game capture overlays (Overwolf, Outplayed, ShadowPlay, Game Bar)
    // from classifying WebView2's swapchain as a fullscreen Direct3D game or hooking into it.
    unsafe {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            format!(
                "{} --disable-direct-composition-video-overlays --disable-features=HardwareMediaKeyHandling,CalculateNativeWinOcclusion",
                std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default(),
            ),
        );
        std::env::set_var("SHIM_MCCOMPAT", "0x800000001");
        std::env::set_var("DISABLE_VULKAN_OW_OVERLAY_1", "1");
        std::env::set_var("OW_DISABLE_HOOK", "1");
        std::env::set_var("OVERWOLF_DISABLE_INJECTION", "1");
    }
    dlc_lib::run()
}
