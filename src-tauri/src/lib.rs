mod runtime;

mod providers {
    pub mod provider;
    pub mod ollama;
    pub mod lm_studio;
    pub mod llama_cpp;
    pub mod jan;
    pub mod gpt4all;
    pub mod localai;
    pub mod koboldcpp;
    pub mod llamafile;
    pub mod vllm;
    pub mod mlx;
    pub mod manager;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Runtime discovery
            runtime::discover_runtimes,

            // Provider Manager
            providers::manager::provider_get_info,
            providers::manager::provider_connect,
            providers::manager::provider_get_models,
            providers::manager::provider_start,
            providers::manager::provider_stop,
            providers::manager::provider_download_model,
            providers::manager::provider_delete_model,
            providers::manager::provider_chat,

            // Legacy provider commands
            providers::ollama::get_ollama_models,
            providers::lm_studio::get_lm_studio_models,
            providers::llama_cpp::get_llama_cpp_models,
            providers::jan::get_jan_models,
            providers::gpt4all::get_gpt4all_models,
            providers::localai::get_localai_models,
            providers::koboldcpp::get_koboldcpp_models,
            providers::llamafile::get_llamafile_models,
            providers::vllm::get_vllm_models,
            providers::mlx::get_mlx_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}